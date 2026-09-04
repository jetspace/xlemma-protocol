// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";

/// @notice Append-only registry of content-addressed proof and certificate roots.
/// @dev Records are never deleted. Corrections create new records and explicit
///      supersession edges; status transitions remain visible in event history.
contract ProofRegistry is AccessControl {
    bytes32 public constant CERTIFIER_ROLE = keccak256("CERTIFIER_ROLE");
    bytes32 public constant QUARANTINE_ROLE = keccak256("QUARANTINE_ROLE");

    enum FormalStatus {
        Unset,
        Committed,
        Reproduced,
        Certified,
        Rejected,
        Divergent,
        Quarantined,
        Revoked,
        Superseded
    }

    struct Record {
        bytes32 theoryId;
        bytes32 claimId;
        bytes32 proofId;
        bytes32 artifactRoot;
        bytes32 policyId;
        bytes32 certificateRoot;
        bytes32 parentRecordId;
        uint64 createdAt;
        uint64 challengeEndsAt;
        FormalStatus status;
    }

    mapping(bytes32 => Record) public records;

    event RecordCommitted(bytes32 indexed recordId, bytes32 indexed claimId, bytes32 artifactRoot);
    event CertificateAttached(bytes32 indexed recordId, bytes32 certificateRoot, FormalStatus status);
    event RecordQuarantined(bytes32 indexed recordId, bytes32 evidenceRoot);
    event RecordRevoked(bytes32 indexed recordId, bytes32 evidenceRoot);
    event RecordSuperseded(bytes32 indexed oldRecordId, bytes32 indexed newRecordId);

    error RecordExists();
    error RecordMissing();
    error InvalidInput();
    error InvalidTransition();

    constructor(address administrator) {
        if (administrator == address(0)) revert InvalidInput();
        _grantRole(DEFAULT_ADMIN_ROLE, administrator);
        _grantRole(CERTIFIER_ROLE, administrator);
        _grantRole(QUARANTINE_ROLE, administrator);
    }

    function getRecord(bytes32 recordId) external view returns (Record memory) {
        return records[recordId];
    }

    function commitRecord(bytes32 recordId, Record calldata proposed) external {
        if (recordId == bytes32(0) || proposed.claimId == bytes32(0) || proposed.artifactRoot == bytes32(0)) {
            revert InvalidInput();
        }
        if (records[recordId].createdAt != 0) revert RecordExists();
        if (proposed.status != FormalStatus.Committed) revert InvalidTransition();

        records[recordId] = Record({
            theoryId: proposed.theoryId,
            claimId: proposed.claimId,
            proofId: proposed.proofId,
            artifactRoot: proposed.artifactRoot,
            policyId: proposed.policyId,
            certificateRoot: bytes32(0),
            parentRecordId: proposed.parentRecordId,
            createdAt: uint64(block.timestamp),
            challengeEndsAt: 0,
            status: FormalStatus.Committed
        });
        emit RecordCommitted(recordId, proposed.claimId, proposed.artifactRoot);
    }

    /// @notice Attaches the first independent reproduction certificate.
    ///         Later assurance upgrades SHOULD create a child record so the
    ///         original certificate remains immutable and independently auditable.
    function attachCertificate(
        bytes32 recordId,
        bytes32 certificateRoot,
        FormalStatus status,
        uint64 challengeEndsAt
    ) external onlyRole(CERTIFIER_ROLE) {
        Record storage record = records[recordId];
        if (record.createdAt == 0) revert RecordMissing();
        if (record.status != FormalStatus.Committed || record.certificateRoot != bytes32(0)) {
            revert InvalidTransition();
        }
        if (
            certificateRoot == bytes32(0) ||
            (
                status != FormalStatus.Reproduced &&
                status != FormalStatus.Certified &&
                status != FormalStatus.Rejected &&
                status != FormalStatus.Divergent
            )
        ) revert InvalidTransition();
        record.certificateRoot = certificateRoot;
        record.status = status;
        record.challengeEndsAt = challengeEndsAt;
        emit CertificateAttached(recordId, certificateRoot, status);
    }

    function quarantine(bytes32 recordId, bytes32 evidenceRoot)
        external
        onlyRole(QUARANTINE_ROLE)
    {
        Record storage record = records[recordId];
        if (record.createdAt == 0) revert RecordMissing();
        if (evidenceRoot == bytes32(0)) revert InvalidInput();
        record.status = FormalStatus.Quarantined;
        emit RecordQuarantined(recordId, evidenceRoot);
    }

    function revoke(bytes32 recordId, bytes32 evidenceRoot)
        external
        onlyRole(QUARANTINE_ROLE)
    {
        Record storage record = records[recordId];
        if (record.createdAt == 0) revert RecordMissing();
        if (evidenceRoot == bytes32(0)) revert InvalidInput();
        record.status = FormalStatus.Revoked;
        emit RecordRevoked(recordId, evidenceRoot);
    }

    function supersede(bytes32 oldRecordId, bytes32 newRecordId)
        external
        onlyRole(CERTIFIER_ROLE)
    {
        if (oldRecordId == newRecordId) revert InvalidInput();
        Record storage oldRecord = records[oldRecordId];
        Record storage newRecord = records[newRecordId];
        if (oldRecord.createdAt == 0 || newRecord.createdAt == 0) revert RecordMissing();
        if (oldRecord.status == FormalStatus.Superseded || newRecord.parentRecordId != bytes32(0)) {
            revert InvalidTransition();
        }
        oldRecord.status = FormalStatus.Superseded;
        newRecord.parentRecordId = oldRecordId;
        emit RecordSuperseded(oldRecordId, newRecordId);
    }
}
