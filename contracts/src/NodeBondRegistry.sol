// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @notice Neutral collateral registry. Stake establishes eligibility, not truth-weight.
contract NodeBondRegistry is AccessControl, ReentrancyGuard {
    using SafeERC20 for IERC20;

    bytes32 public constant SLASHER_ROLE = keccak256("SLASHER_ROLE");

    struct Node {
        address owner;
        bytes32 operatorClusterId;
        bytes32 rolesRoot;
        uint256 bonded;
        uint256 pendingUnbond;
        uint64 unbondAvailableAt;
        bool active;
    }

    IERC20 public immutable bondAsset;
    uint64 public immutable unbondDelay;
    mapping(bytes32 => Node) public nodes;

    event Registered(bytes32 indexed nodeId, address indexed owner, bytes32 operatorClusterId);
    event RolesUpdated(bytes32 indexed nodeId, bytes32 rolesRoot);
    event ActiveStateChanged(bytes32 indexed nodeId, bool active);
    event Bonded(bytes32 indexed nodeId, uint256 amount);
    event UnbondRequested(bytes32 indexed nodeId, uint256 amount, uint64 availableAt);
    event UnbondWithdrawn(bytes32 indexed nodeId, uint256 amount);
    event Slashed(bytes32 indexed nodeId, uint256 amount, bytes32 evidenceRoot, address recipient);

    error Unauthorized();
    error InvalidAddress();
    error InvalidAmount();
    error InvalidState();
    error DelayOpen();

    constructor(IERC20 bondAsset_, address administrator, uint64 unbondDelay_) {
        if (address(bondAsset_) == address(0) || administrator == address(0)) {
            revert InvalidAddress();
        }
        bondAsset = bondAsset_;
        unbondDelay = unbondDelay_;
        _grantRole(DEFAULT_ADMIN_ROLE, administrator);
        _grantRole(SLASHER_ROLE, administrator);
    }

    function getNode(bytes32 nodeId) external view returns (Node memory) {
        return nodes[nodeId];
    }

    function register(bytes32 nodeId, bytes32 operatorClusterId, bytes32 rolesRoot) external {
        if (nodeId == bytes32(0) || operatorClusterId == bytes32(0) || rolesRoot == bytes32(0)) {
            revert InvalidState();
        }
        Node storage node = nodes[nodeId];
        if (node.owner == address(0)) {
            node.owner = msg.sender;
            node.operatorClusterId = operatorClusterId;
        } else {
            if (node.owner != msg.sender) revert Unauthorized();
            // Operator identity is immutable for a NodeID. A changed operator
            // must register a new NodeID and rebuild reputation.
            if (node.operatorClusterId != operatorClusterId) revert InvalidState();
        }
        node.rolesRoot = rolesRoot;
        node.active = true;
        emit Registered(nodeId, msg.sender, operatorClusterId);
    }

    function updateRoles(bytes32 nodeId, bytes32 rolesRoot) external {
        Node storage node = nodes[nodeId];
        if (node.owner != msg.sender) revert Unauthorized();
        if (rolesRoot == bytes32(0)) revert InvalidState();
        node.rolesRoot = rolesRoot;
        emit RolesUpdated(nodeId, rolesRoot);
    }

    function setActive(bytes32 nodeId, bool active) external {
        Node storage node = nodes[nodeId];
        if (node.owner != msg.sender) revert Unauthorized();
        if (active && node.pendingUnbond != 0) revert InvalidState();
        node.active = active;
        emit ActiveStateChanged(nodeId, active);
    }

    function bond(bytes32 nodeId, uint256 amount) external nonReentrant {
        Node storage node = nodes[nodeId];
        if (node.owner != msg.sender) revert Unauthorized();
        if (amount == 0) revert InvalidAmount();
        bondAsset.safeTransferFrom(msg.sender, address(this), amount);
        node.bonded += amount;
        emit Bonded(nodeId, amount);
    }

    function requestUnbond(bytes32 nodeId, uint256 amount) external {
        Node storage node = nodes[nodeId];
        if (node.owner != msg.sender) revert Unauthorized();
        if (amount == 0 || amount > node.bonded || node.pendingUnbond != 0) {
            revert InvalidAmount();
        }
        node.active = false;
        node.pendingUnbond = amount;
        node.unbondAvailableAt = uint64(block.timestamp) + unbondDelay;
        emit ActiveStateChanged(nodeId, false);
        emit UnbondRequested(nodeId, amount, node.unbondAvailableAt);
    }

    function withdrawUnbond(bytes32 nodeId) external nonReentrant {
        Node storage node = nodes[nodeId];
        if (node.owner != msg.sender) revert Unauthorized();
        if (node.pendingUnbond == 0) revert InvalidState();
        if (block.timestamp < node.unbondAvailableAt) revert DelayOpen();
        uint256 amount = node.pendingUnbond;
        node.pendingUnbond = 0;
        node.unbondAvailableAt = 0;
        node.bonded -= amount;
        bondAsset.safeTransfer(msg.sender, amount);
        emit UnbondWithdrawn(nodeId, amount);
    }

    /// @dev Invoke only for objectively provable misconduct such as equivocation,
    /// fabricated execution evidence, or false custody attestations—not honest dissent.
    function slash(bytes32 nodeId, uint256 amount, bytes32 evidenceRoot, address recipient)
        external
        onlyRole(SLASHER_ROLE)
        nonReentrant
    {
        Node storage node = nodes[nodeId];
        if (
            amount == 0 || amount > node.bonded || evidenceRoot == bytes32(0) ||
            recipient == address(0)
        ) revert InvalidAmount();
        node.bonded -= amount;
        if (node.pendingUnbond > node.bonded) {
            node.pendingUnbond = node.bonded;
        }
        node.active = false;
        bondAsset.safeTransfer(recipient, amount);
        emit ActiveStateChanged(nodeId, false);
        emit Slashed(nodeId, amount, evidenceRoot, recipient);
    }
}
