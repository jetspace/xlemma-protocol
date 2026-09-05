// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

interface IDiscoveryEvidenceRegistry {
    function isFinalFor(bytes32 certificate, bytes32 claim, bytes32 artifact, bytes32 policy)
        external
        view
        returns (bool);
}

/// @notice Funded discovery settlement with isolated categories and independent plan approval.
/// @dev Evidence registries and operator-control mappings are qualified deployment dependencies.
///      This contract never establishes mathematical or experimental truth by voting.
contract DiscoveryRoundEscrow is AccessControl, ReentrancyGuard {
    using SafeERC20 for IERC20;
    bytes32 public constant PLANNER_ROLE = keccak256("PLANNER_ROLE");
    bytes32 public constant REVIEWER_ROLE = keccak256("REVIEWER_ROLE");
    bytes32 public constant WATCHER_ROLE = keccak256("WATCHER_ROLE");
    bytes32 public constant RESOLVER_ROLE = keccak256("RESOLVER_ROLE");
    uint64 public constant MINIMUM_PLAN_DELAY = 1 hours;
    uint256 public constant MAX_ITEMS = 256;
    IERC20 public immutable asset;
    IDiscoveryEvidenceRegistry public immutable evidenceRegistry;

    struct Round {
        bytes32 policyRoot;
        uint64 opensAt;
        uint64 submissionsCloseAt;
        uint64 appealEndsAt;
        uint64 expiresAt;
        bool held;
        bool paid;
        bool expired;
        bytes32 planId;
        bytes32 holdRoot;
        uint256[7] solverCaps;
        uint256[7] reviewCaps;
        uint256[7] deposits;
        uint256[7] spent;
    }

    struct Item {
        uint8 category;
        bool completedReview;
        address recipient;
        uint256 amount;
        bytes32 certificate;
        bytes32 claim;
        bytes32 artifact;
        bytes32 policy;
        bytes32 workRoot;
    }

    struct Plan {
        bytes32 roundId;
        bytes32 commitment;
        address proposer;
        bytes32 proposerCluster;
        uint64 executableAt;
        address[] reviewers;
        bytes32[] reviewerClusters;
        Item[] items;
    }

    mapping(bytes32 => Round) private rounds;
    mapping(bytes32 => Plan) private plans;
    mapping(address => bytes32) public operatorCluster;
    mapping(bytes32 => mapping(uint8 => mapping(address => uint256))) public contributions;
    mapping(bytes32 => mapping(uint8 => mapping(address => bool))) public refunded;

    struct ResolutionVotes {
        address[] reviewers;
        bytes32[] clusters;
    }
    mapping(bytes32 => mapping(bytes32 => ResolutionVotes)) private resolutionVotes;
    mapping(bytes32 => mapping(bytes32 => bool)) private usedHolds;
    uint256 public fundingSequence;
    mapping(bytes32 => bool) public completedWorkPaid;

    event RoundCreated(bytes32 indexed roundId, bytes32 indexed policyRoot);
    event Funded(
        bytes32 indexed roundId,
        uint8 indexed category,
        address indexed funder,
        uint256 amount,
        bytes32 mandateRoot,
        uint256 sequence
    );
    event PlanProposed(bytes32 indexed roundId, bytes32 indexed planId, bytes32 commitment, uint64 executableAt);
    event PlanApproved(bytes32 indexed planId, address indexed reviewer, bytes32 cluster);
    event Held(bytes32 indexed roundId, bytes32 evidenceRoot);
    event HoldResolved(bytes32 indexed roundId, bytes32 evidenceRoot, bytes32 resolutionRoot);
    event Settled(bytes32 indexed roundId, bytes32 indexed planId, bytes32 commitment, uint256 total);
    event Expired(bytes32 indexed roundId);
    event Refunded(bytes32 indexed roundId, uint8 indexed category, address indexed funder, uint256 amount);

    error Invalid();
    error Closed();
    error InsufficientFunding();
    error EvidenceNotFinal();
    error WorkAlreadyPaid();
    error UnsupportedAsset();

    constructor(IERC20 token, IDiscoveryEvidenceRegistry registry, address administrator) {
        if (
            address(token) == address(0) || address(registry) == address(0) || administrator == address(0)
                || IERC20Metadata(address(token)).decimals() != 6
        ) revert Invalid();
        asset = token;
        evidenceRegistry = registry;
        _grantRole(DEFAULT_ADMIN_ROLE, administrator);
    }

    function setOperatorCluster(address operator, bytes32 cluster) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (operator == address(0) || cluster == bytes32(0)) revert Invalid();
        operatorCluster[operator] = cluster;
    }

    function createRound(
        bytes32 roundId,
        bytes32 policyRoot,
        uint64 opensAt,
        uint64 submissionsCloseAt,
        uint64 appealEndsAt,
        uint64 expiresAt,
        uint16 foundationalBps,
        uint256[7] calldata solverCaps,
        uint256[7] calldata reviewCaps
    ) external onlyRole(PLANNER_ROLE) {
        if (
            roundId == bytes32(0) || policyRoot == bytes32(0) || rounds[roundId].policyRoot != bytes32(0)
                || block.timestamp >= opensAt || opensAt >= submissionsCloseAt || submissionsCloseAt >= appealEndsAt
                || uint256(appealEndsAt) + MINIMUM_PLAN_DELAY >= expiresAt || foundationalBps == 0
                || foundationalBps > 10_000
        ) revert Invalid();
        uint256 total = 0;
        for (uint256 i = 0; i < 7; ++i) {
            total += solverCaps[i];
        }
        if (total == 0 || solverCaps[0] * 10_000 < total * foundationalBps) revert Invalid();
        Round storage r = rounds[roundId];
        r.policyRoot = policyRoot;
        r.opensAt = opensAt;
        r.submissionsCloseAt = submissionsCloseAt;
        r.appealEndsAt = appealEndsAt;
        r.expiresAt = expiresAt;
        r.solverCaps = solverCaps;
        r.reviewCaps = reviewCaps;
        emit RoundCreated(roundId, policyRoot);
    }

    function fund(bytes32 roundId, uint8 category, uint256 amount, bytes32 mandateRoot) external nonReentrant {
        Round storage r = rounds[roundId];
        if (
            r.policyRoot == bytes32(0) || block.timestamp >= r.opensAt || category >= 7 || amount == 0
                || mandateRoot == bytes32(0)
        ) revert Invalid();
        uint256 beforeBalance = asset.balanceOf(address(this));
        asset.safeTransferFrom(msg.sender, address(this), amount);
        if (asset.balanceOf(address(this)) - beforeBalance != amount) revert UnsupportedAsset();
        r.deposits[category] += amount;
        contributions[roundId][category][msg.sender] += amount;
        emit Funded(roundId, category, msg.sender, amount, mandateRoot, fundingSequence++);
    }

    function proposePlan(bytes32 roundId, bytes32 planId, Item[] calldata items) external onlyRole(PLANNER_ROLE) {
        Round storage r = rounds[roundId];
        if (
            r.policyRoot == bytes32(0) || r.planId != bytes32(0) || r.paid || r.expired || r.held
                || block.timestamp < r.appealEndsAt || block.timestamp + MINIMUM_PLAN_DELAY >= r.expiresAt
                || planId == bytes32(0) || plans[planId].commitment != bytes32(0) || items.length == 0
                || items.length > MAX_ITEMS || operatorCluster[msg.sender] == bytes32(0)
        ) revert Invalid();
        uint256[7] memory rewards = [uint256(0), 0, 0, 0, 0, 0, 0];
        uint256[7] memory reviews = [uint256(0), 0, 0, 0, 0, 0, 0];
        for (uint256 i = 0; i < items.length; ++i) {
            Item calldata item = items[i];
            if (
                item.category >= 7 || item.recipient == address(0) || item.recipient == address(this)
                    || item.amount == 0 || item.workRoot == bytes32(0)
            ) revert Invalid();
            if (item.completedReview) {
                reviews[item.category] += item.amount;
            } else {
                if (
                    item.certificate == bytes32(0) || item.claim == bytes32(0) || item.artifact == bytes32(0)
                        || item.policy == bytes32(0)
                ) revert Invalid();
                rewards[item.category] += item.amount;
            }
        }
        for (uint256 i = 0; i < 7; ++i) {
            if (rewards[i] > r.solverCaps[i] || reviews[i] > r.reviewCaps[i] || rewards[i] + reviews[i] > r.deposits[i])
            {
                revert InsufficientFunding();
            }
        }
        bytes32 commitment = keccak256(abi.encode(block.chainid, address(this), roundId, planId, r.policyRoot, items));
        Plan storage p = plans[planId];
        p.roundId = roundId;
        p.commitment = commitment;
        p.proposer = msg.sender;
        p.proposerCluster = operatorCluster[msg.sender];
        p.executableAt = uint64(block.timestamp) + MINIMUM_PLAN_DELAY;
        for (uint256 i = 0; i < items.length; ++i) {
            p.items.push(items[i]);
        }
        r.planId = planId;
        emit PlanProposed(roundId, planId, commitment, p.executableAt);
    }

    function approvePlan(bytes32 planId, bytes32 commitment) external onlyRole(REVIEWER_ROLE) {
        Plan storage p = plans[planId];
        Round storage r = rounds[p.roundId];
        bytes32 cluster = operatorCluster[msg.sender];
        if (
            p.commitment == bytes32(0) || p.commitment != commitment || r.paid || r.expired || r.held
                || cluster == bytes32(0) || cluster == p.proposerCluster || p.reviewers.length >= 8
        ) revert Invalid();
        for (uint256 i = 0; i < p.reviewerClusters.length; ++i) {
            if (p.reviewerClusters[i] == cluster) revert Invalid();
        }
        p.reviewers.push(msg.sender);
        p.reviewerClusters.push(cluster);
        emit PlanApproved(planId, msg.sender, cluster);
    }

    function hold(bytes32 roundId, bytes32 evidenceRoot) external onlyRole(WATCHER_ROLE) {
        Round storage r = rounds[roundId];
        if (
            r.policyRoot == bytes32(0) || r.paid || r.expired || r.held || evidenceRoot == bytes32(0)
                || usedHolds[roundId][evidenceRoot]
        ) revert Invalid();
        usedHolds[roundId][evidenceRoot] = true;
        r.held = true;
        r.holdRoot = evidenceRoot;
        emit Held(roundId, evidenceRoot);
    }

    function resolveHold(bytes32 roundId, bytes32 resolutionRoot) external onlyRole(RESOLVER_ROLE) {
        Round storage r = rounds[roundId];
        bytes32 cluster = operatorCluster[msg.sender];
        if (
            !r.held || cluster == bytes32(0) || resolutionRoot == bytes32(0)
                || (r.planId != bytes32(0) && cluster == plans[r.planId].proposerCluster)
        ) revert Invalid();
        bytes32 resolutionKey = keccak256(abi.encode(r.holdRoot, resolutionRoot));
        ResolutionVotes storage votes = resolutionVotes[roundId][resolutionKey];
        if (votes.reviewers.length >= 8) revert Invalid();
        for (uint256 i = 0; i < votes.reviewers.length; ++i) {
            if (votes.reviewers[i] == msg.sender || votes.clusters[i] == cluster) revert Invalid();
        }
        votes.reviewers.push(msg.sender);
        votes.clusters.push(cluster);
        uint256 approvals = 0;
        for (uint256 i = 0; i < votes.reviewers.length; ++i) {
            if (
                hasRole(RESOLVER_ROLE, votes.reviewers[i]) && operatorCluster[votes.reviewers[i]] == votes.clusters[i]
                    && (r.planId == bytes32(0) || votes.clusters[i] != plans[r.planId].proposerCluster)
            ) ++approvals;
        }
        if (approvals >= 2) {
            r.held = false;
            if (r.planId != bytes32(0)) plans[r.planId].executableAt = uint64(block.timestamp) + MINIMUM_PLAN_DELAY;
            emit HoldResolved(roundId, r.holdRoot, resolutionRoot);
        }
    }

    function executePlan(bytes32 roundId) external nonReentrant {
        Round storage r = rounds[roundId];
        Plan storage p = plans[r.planId];
        if (
            r.paid || r.expired || r.held || p.commitment == bytes32(0) || block.timestamp < p.executableAt
                || block.timestamp >= r.expiresAt
        ) revert Closed();
        uint256 approvals = 0;
        for (uint256 i = 0; i < p.reviewers.length; ++i) {
            if (hasRole(REVIEWER_ROLE, p.reviewers[i]) && operatorCluster[p.reviewers[i]] == p.reviewerClusters[i]) {
                ++approvals;
            }
        }
        if (approvals < 2 || !hasRole(PLANNER_ROLE, p.proposer) || operatorCluster[p.proposer] != p.proposerCluster) {
            revert Invalid();
        }
        uint256 total = 0;
        for (uint256 i = 0; i < p.items.length; ++i) {
            Item storage item = p.items[i];
            if (item.completedReview) {
                if (completedWorkPaid[item.workRoot]) revert WorkAlreadyPaid();
                completedWorkPaid[item.workRoot] = true;
            }
            if (
                !item.completedReview
                    && !evidenceRegistry.isFinalFor(item.certificate, item.claim, item.artifact, item.policy)
            ) revert EvidenceNotFinal();
            r.spent[item.category] += item.amount;
            total += item.amount;
        }
        r.paid = true;
        uint256 beforeBalance = asset.balanceOf(address(this));
        for (uint256 i = 0; i < p.items.length; ++i) {
            transferExact(p.items[i].recipient, p.items[i].amount);
        }
        if (beforeBalance - asset.balanceOf(address(this)) != total) revert UnsupportedAsset();
        emit Settled(roundId, r.planId, p.commitment, total);
    }

    function expire(bytes32 roundId) external {
        Round storage r = rounds[roundId];
        if (r.policyRoot == bytes32(0) || r.paid || r.expired || block.timestamp < r.expiresAt) revert Closed();
        r.expired = true;
        emit Expired(roundId);
    }

    function refund(bytes32 roundId, uint8 category) external nonReentrant {
        Round storage r = rounds[roundId];
        if ((!r.paid && !r.expired) || category >= 7 || refunded[roundId][category][msg.sender]) revert Closed();
        uint256 contribution = contributions[roundId][category][msg.sender];
        if (contribution == 0) revert Invalid();
        refunded[roundId][category][msg.sender] = true;
        uint256 amount = Math.mulDiv(contribution, r.deposits[category] - r.spent[category], r.deposits[category]);
        if (amount > 0) transferExact(msg.sender, amount);
        emit Refunded(roundId, category, msg.sender, amount);
    }

    function round(bytes32 id) external view returns (Round memory) {
        return rounds[id];
    }

    function transferExact(address recipient, uint256 amount) private {
        uint256 beforeBalance = asset.balanceOf(recipient);
        asset.safeTransfer(recipient, amount);
        if (asset.balanceOf(recipient) - beforeBalance != amount) revert UnsupportedAsset();
    }

    function planCommitment(bytes32 id) external view returns (bytes32) {
        return plans[id].commitment;
    }
}
