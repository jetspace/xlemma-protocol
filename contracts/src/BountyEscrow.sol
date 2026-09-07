// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

interface IPoIRCertificateRegistry {
    function isFinalFor(bytes32 certificateId, bytes32 claimId, bytes32 artifactRoot, bytes32 policyId)
        external
        view
        returns (bool);
}

/// @notice Reverse-direction research bounty escrow, separate from ordinary x402 payment.
contract BountyEscrow is AccessControl, ReentrancyGuard {
    using SafeERC20 for IERC20;

    bytes32 public constant CERTIFICATE_ROLE = keccak256("CERTIFICATE_ROLE");
    uint64 public constant MINIMUM_POST_ACCEPTANCE_DELAY = 1 hours;

    struct Bounty {
        address sponsor;
        IERC20 asset;
        uint256 reward;
        bytes32 claimId;
        bytes32 policyId;
        uint64 deadline;
        uint64 postAcceptanceDelay;
        uint64 releaseAfter;
        bool paid;
        bool refunded;
        address acceptedSolver;
        bytes32 acceptedArtifactRoot;
        bytes32 acceptedCertificate;
    }

    IPoIRCertificateRegistry public immutable certificateRegistry;
    mapping(bytes32 => Bounty) public bounties;
    mapping(bytes32 => mapping(address => bytes32)) public commitments;

    event BountyCreated(bytes32 indexed bountyId, bytes32 indexed claimId, uint256 reward);
    event SolutionCommitted(bytes32 indexed bountyId, address indexed solver, bytes32 commitment);
    event SolutionAccepted(
        bytes32 indexed bountyId, address indexed solver, bytes32 certificateId, uint64 releaseAfter
    );
    event BountyPaid(bytes32 indexed bountyId, address indexed solver, uint256 reward);
    event BountyRefunded(bytes32 indexed bountyId, address indexed sponsor, uint256 reward);

    error InvalidState();
    error InvalidInput();
    error InvalidReveal();
    error CertificateNotFinal();
    error ReleaseDelayOpen();
    error UnsupportedAssetBehavior();

    constructor(IPoIRCertificateRegistry certificateRegistry_, address administrator) {
        if (address(certificateRegistry_) == address(0) || administrator == address(0)) {
            revert InvalidInput();
        }
        certificateRegistry = certificateRegistry_;
        _grantRole(DEFAULT_ADMIN_ROLE, administrator);
        _grantRole(CERTIFICATE_ROLE, administrator);
    }

    function deriveBountyId(
        address sponsor,
        IERC20 asset,
        uint256 reward,
        bytes32 claimId,
        bytes32 policyId,
        uint64 deadline,
        uint64 postAcceptanceDelay
    ) public pure returns (bytes32) {
        return keccak256(abi.encode(sponsor, asset, reward, claimId, policyId, deadline, postAcceptanceDelay));
    }

    function create(
        bytes32 bountyId,
        IERC20 asset,
        uint256 reward,
        bytes32 claimId,
        bytes32 policyId,
        uint64 deadline,
        uint64 postAcceptanceDelay
    ) external nonReentrant {
        if (
            bountyId == bytes32(0)
                || bountyId
                    != deriveBountyId(msg.sender, asset, reward, claimId, policyId, deadline, postAcceptanceDelay)
                || address(asset) == address(0) || claimId == bytes32(0) || policyId == bytes32(0)
                || bounties[bountyId].sponsor != address(0) || reward == 0 || deadline <= block.timestamp
                || postAcceptanceDelay < MINIMUM_POST_ACCEPTANCE_DELAY
        ) revert InvalidInput();

        uint256 beforeBalance = asset.balanceOf(address(this));
        asset.safeTransferFrom(msg.sender, address(this), reward);
        if (asset.balanceOf(address(this)) != beforeBalance + reward) {
            revert UnsupportedAssetBehavior();
        }
        bounties[bountyId] = Bounty({
            sponsor: msg.sender,
            asset: asset,
            reward: reward,
            claimId: claimId,
            policyId: policyId,
            deadline: deadline,
            postAcceptanceDelay: postAcceptanceDelay,
            releaseAfter: 0,
            paid: false,
            refunded: false,
            acceptedSolver: address(0),
            acceptedArtifactRoot: bytes32(0),
            acceptedCertificate: bytes32(0)
        });
        emit BountyCreated(bountyId, claimId, reward);
    }

    function commitSolution(bytes32 bountyId, bytes32 commitment) external {
        Bounty storage bounty = bounties[bountyId];
        if (
            bounty.sponsor == address(0) || block.timestamp > bounty.deadline || commitment == bytes32(0)
                || commitments[bountyId][msg.sender] != bytes32(0)
        ) revert InvalidState();
        commitments[bountyId][msg.sender] = commitment;
        emit SolutionCommitted(bountyId, msg.sender, commitment);
    }

    function acceptCertifiedSolution(
        bytes32 bountyId,
        address solver,
        bytes32 artifactRoot,
        bytes32 salt,
        bytes32 certificateId
    ) external onlyRole(CERTIFICATE_ROLE) {
        Bounty storage bounty = bounties[bountyId];
        if (
            bounty.sponsor == address(0) || bounty.acceptedSolver != address(0) || bounty.paid || bounty.refunded
                || block.timestamp > bounty.deadline || solver == address(0) || artifactRoot == bytes32(0)
                || certificateId == bytes32(0)
        ) revert InvalidState();

        bytes32 expected = keccak256(abi.encode(bountyId, artifactRoot, salt, solver));
        if (commitments[bountyId][solver] != expected) revert InvalidReveal();
        if (!certificateRegistry.isFinalFor(certificateId, bounty.claimId, artifactRoot, bounty.policyId)) {
            revert CertificateNotFinal();
        }

        bounty.acceptedSolver = solver;
        bounty.acceptedArtifactRoot = artifactRoot;
        bounty.acceptedCertificate = certificateId;
        bounty.releaseAfter = uint64(block.timestamp) + bounty.postAcceptanceDelay;
        emit SolutionAccepted(bountyId, solver, certificateId, bounty.releaseAfter);
    }

    function release(bytes32 bountyId) external nonReentrant {
        Bounty storage bounty = bounties[bountyId];
        if (bounty.acceptedSolver == address(0) || bounty.paid || bounty.refunded) {
            revert InvalidState();
        }
        if (block.timestamp < bounty.releaseAfter) revert ReleaseDelayOpen();
        if (!certificateRegistry.isFinalFor(
                bounty.acceptedCertificate, bounty.claimId, bounty.acceptedArtifactRoot, bounty.policyId
            )) {
            revert CertificateNotFinal();
        }

        bounty.paid = true;
        _pushExact(bounty.asset, bounty.acceptedSolver, bounty.reward);
        emit BountyPaid(bountyId, bounty.acceptedSolver, bounty.reward);
    }

    function refundExpired(bytes32 bountyId) external nonReentrant {
        Bounty storage bounty = bounties[bountyId];
        if (
            bounty.sponsor == address(0) || msg.sender != bounty.sponsor || bounty.acceptedSolver != address(0)
                || bounty.paid || bounty.refunded || block.timestamp <= bounty.deadline
        ) revert InvalidState();
        bounty.refunded = true;
        _pushExact(bounty.asset, bounty.sponsor, bounty.reward);
        emit BountyRefunded(bountyId, bounty.sponsor, bounty.reward);
    }

    /// @notice Recovers an accepted bounty after its certificate loses final
    ///         status. The original acceptance delay doubles as a safety window
    ///         so a transient registry condition cannot trigger an early refund.
    function refundInvalidatedAcceptance(bytes32 bountyId) external nonReentrant {
        Bounty storage bounty = bounties[bountyId];
        if (
            bounty.sponsor == address(0) || msg.sender != bounty.sponsor || bounty.acceptedSolver == address(0)
                || bounty.paid || bounty.refunded || block.timestamp < bounty.releaseAfter
                || certificateRegistry.isFinalFor(
                    bounty.acceptedCertificate, bounty.claimId, bounty.acceptedArtifactRoot, bounty.policyId
                )
        ) revert InvalidState();
        bounty.refunded = true;
        _pushExact(bounty.asset, bounty.sponsor, bounty.reward);
        emit BountyRefunded(bountyId, bounty.sponsor, bounty.reward);
    }

    function _pushExact(IERC20 asset, address recipient, uint256 amount) internal {
        uint256 beforeEscrow = asset.balanceOf(address(this));
        uint256 beforeRecipient = asset.balanceOf(recipient);
        asset.safeTransfer(recipient, amount);
        if (
            asset.balanceOf(address(this)) + amount != beforeEscrow
                || asset.balanceOf(recipient) != beforeRecipient + amount
        ) revert UnsupportedAssetBehavior();
    }
}
