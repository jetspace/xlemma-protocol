// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";

/// @notice Append-only settlement-layer projection of an xLemma research object.
/// @dev The registry commits ordering and content roots only. It cannot decide
///      research validity, rewrite origin, interpret rights, or transfer custody.
contract ResearchCommitmentRegistry is AccessControl {
    bytes32 public constant COMMITTER_ROLE = keccak256("COMMITTER_ROLE");

    struct Commitment {
        bytes32 researcherId;
        bytes32 claimId;
        bytes32 artifactRoot;
        bytes32 protocolPolicyId;
        bytes32 committeeAssignmentRoot;
        bytes32 rightsManifestRoot;
        bytes32 contributionSplitRoot;
        bytes32 parentCommitmentId;
        uint64 createdAt;
        bytes32 supersededBy;
    }

    mapping(bytes32 => Commitment) public commitments;

    event ResearchCommitted(
        bytes32 indexed commitmentId, bytes32 indexed researcherId, bytes32 indexed claimId, bytes32 artifactRoot
    );
    event ResearchSuperseded(bytes32 indexed oldCommitmentId, bytes32 indexed newCommitmentId);

    error CommitmentExists();
    error CommitmentMissing();
    error InvalidInput();
    error InvalidTransition();

    constructor(address administrator) {
        if (administrator == address(0)) revert InvalidInput();
        _grantRole(DEFAULT_ADMIN_ROLE, administrator);
        _grantRole(COMMITTER_ROLE, administrator);
    }

    function getCommitment(bytes32 commitmentId) external view returns (Commitment memory) {
        return commitments[commitmentId];
    }

    function deriveCommitmentId(Commitment calldata proposed) public pure returns (bytes32) {
        return keccak256(
            abi.encode(
                proposed.researcherId,
                proposed.claimId,
                proposed.artifactRoot,
                proposed.protocolPolicyId,
                proposed.committeeAssignmentRoot,
                proposed.rightsManifestRoot,
                proposed.contributionSplitRoot,
                proposed.parentCommitmentId
            )
        );
    }

    function commit(bytes32 commitmentId, Commitment calldata proposed) external onlyRole(COMMITTER_ROLE) {
        if (
            commitmentId == bytes32(0) || commitmentId != deriveCommitmentId(proposed)
                || proposed.researcherId == bytes32(0) || proposed.claimId == bytes32(0)
                || proposed.artifactRoot == bytes32(0) || proposed.protocolPolicyId == bytes32(0)
                || proposed.committeeAssignmentRoot == bytes32(0) || proposed.rightsManifestRoot == bytes32(0)
                || proposed.contributionSplitRoot == bytes32(0) || proposed.createdAt != 0
                || proposed.supersededBy != bytes32(0)
        ) revert InvalidInput();
        if (commitments[commitmentId].createdAt != 0) revert CommitmentExists();
        if (proposed.parentCommitmentId != bytes32(0) && commitments[proposed.parentCommitmentId].createdAt == 0) {
            revert CommitmentMissing();
        }

        commitments[commitmentId] = Commitment({
            researcherId: proposed.researcherId,
            claimId: proposed.claimId,
            artifactRoot: proposed.artifactRoot,
            protocolPolicyId: proposed.protocolPolicyId,
            committeeAssignmentRoot: proposed.committeeAssignmentRoot,
            rightsManifestRoot: proposed.rightsManifestRoot,
            contributionSplitRoot: proposed.contributionSplitRoot,
            parentCommitmentId: proposed.parentCommitmentId,
            createdAt: uint64(block.timestamp),
            supersededBy: bytes32(0)
        });
        emit ResearchCommitted(commitmentId, proposed.researcherId, proposed.claimId, proposed.artifactRoot);
    }

    /// @notice Links an already committed correction without mutating or deleting history.
    function supersede(bytes32 oldCommitmentId, bytes32 newCommitmentId) external onlyRole(COMMITTER_ROLE) {
        if (oldCommitmentId == newCommitmentId) revert InvalidInput();
        Commitment storage oldCommitment = commitments[oldCommitmentId];
        Commitment storage newCommitment = commitments[newCommitmentId];
        if (oldCommitment.createdAt == 0 || newCommitment.createdAt == 0) revert CommitmentMissing();
        if (
            oldCommitment.supersededBy != bytes32(0) || newCommitment.parentCommitmentId != oldCommitmentId
                || newCommitment.researcherId != oldCommitment.researcherId
                || newCommitment.createdAt < oldCommitment.createdAt
        ) revert InvalidTransition();
        oldCommitment.supersededBy = newCommitmentId;
        emit ResearchSuperseded(oldCommitmentId, newCommitmentId);
    }
}
