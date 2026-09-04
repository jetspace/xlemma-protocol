// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";

/// @notice Economic finality for off-chain Proof-of-Independent-Reproduction evidence.
/// @dev This contract records role-authorized certificates. It cannot turn a
///      divergent checker result into a valid theorem; the signed policy and
///      observation roots remain independently reproducible off chain.
contract PoIRCertificateRegistry is AccessControl {
    bytes32 public constant AGGREGATOR_ROLE = keccak256("AGGREGATOR_ROLE");
    bytes32 public constant CHALLENGER_ROLE = keccak256("CHALLENGER_ROLE");
    bytes32 public constant RESOLVER_ROLE = keccak256("RESOLVER_ROLE");

    uint64 public constant MINIMUM_CHALLENGE_PERIOD = 1 hours;

    enum CertificateState {
        Unset,
        Pending,
        Final,
        Challenged,
        Quarantined,
        Rejected
    }

    struct Certificate {
        bytes32 claimId;
        bytes32 proofId;
        bytes32 artifactRoot;
        bytes32 policyId;
        bytes32 observationRoot;
        bytes32 dissentRoot;
        bytes32 challengeEvidenceRoot;
        bytes32 resolutionEvidenceRoot;
        uint64 submittedAt;
        uint64 challengeEndsAt;
        CertificateState state;
    }

    mapping(bytes32 => Certificate) public certificates;

    event Submitted(
        bytes32 indexed certificateId, bytes32 indexed claimId, bytes32 indexed policyId, uint64 challengeEndsAt
    );
    event Challenged(bytes32 indexed certificateId, bytes32 indexed evidenceRoot);
    event ChallengeDismissed(
        bytes32 indexed certificateId, bytes32 indexed resolutionEvidenceRoot, uint64 newChallengeEndsAt
    );
    event Finalized(bytes32 indexed certificateId);
    event Quarantined(bytes32 indexed certificateId, bytes32 indexed evidenceRoot);
    event Rejected(bytes32 indexed certificateId, bytes32 indexed evidenceRoot);

    error InvalidState();
    error InvalidInput();
    error MissingCertificate();
    error ChallengeWindowOpen();
    error ChallengeWindowClosed();

    constructor(address administrator) {
        if (administrator == address(0)) revert InvalidInput();
        _grantRole(DEFAULT_ADMIN_ROLE, administrator);
        _grantRole(AGGREGATOR_ROLE, administrator);
        _grantRole(CHALLENGER_ROLE, administrator);
        _grantRole(RESOLVER_ROLE, administrator);
    }

    function deriveCertificateId(
        bytes32 claimId,
        bytes32 proofId,
        bytes32 artifactRoot,
        bytes32 policyId,
        bytes32 observationRoot,
        bytes32 dissentRoot,
        uint64 challengePeriod
    ) public pure returns (bytes32) {
        return keccak256(
            abi.encode(claimId, proofId, artifactRoot, policyId, observationRoot, dissentRoot, challengePeriod)
        );
    }

    function submit(
        bytes32 certificateId,
        bytes32 claimId,
        bytes32 proofId,
        bytes32 artifactRoot,
        bytes32 policyId,
        bytes32 observationRoot,
        bytes32 dissentRoot,
        uint64 challengePeriod
    ) external onlyRole(AGGREGATOR_ROLE) {
        if (certificates[certificateId].state != CertificateState.Unset) revert InvalidState();
        if (
            certificateId == bytes32(0)
                || certificateId
                    != deriveCertificateId(
                        claimId, proofId, artifactRoot, policyId, observationRoot, dissentRoot, challengePeriod
                    ) || claimId == bytes32(0) || proofId == bytes32(0) || artifactRoot == bytes32(0)
                || policyId == bytes32(0) || observationRoot == bytes32(0) || challengePeriod < MINIMUM_CHALLENGE_PERIOD
        ) revert InvalidInput();

        uint64 submittedAt = uint64(block.timestamp);
        uint64 challengeEndsAt = submittedAt + challengePeriod;
        certificates[certificateId] = Certificate({
            claimId: claimId,
            proofId: proofId,
            artifactRoot: artifactRoot,
            policyId: policyId,
            observationRoot: observationRoot,
            dissentRoot: dissentRoot,
            challengeEvidenceRoot: bytes32(0),
            resolutionEvidenceRoot: bytes32(0),
            submittedAt: submittedAt,
            challengeEndsAt: challengeEndsAt,
            state: CertificateState.Pending
        });
        emit Submitted(certificateId, claimId, policyId, challengeEndsAt);
    }

    function challenge(bytes32 certificateId, bytes32 evidenceRoot) external onlyRole(CHALLENGER_ROLE) {
        Certificate storage certificate = certificates[certificateId];
        if (certificate.state == CertificateState.Unset) revert MissingCertificate();
        if (certificate.state != CertificateState.Pending) revert InvalidState();
        if (block.timestamp > certificate.challengeEndsAt) revert ChallengeWindowClosed();
        if (evidenceRoot == bytes32(0)) revert InvalidInput();
        certificate.challengeEvidenceRoot = evidenceRoot;
        certificate.state = CertificateState.Challenged;
        emit Challenged(certificateId, evidenceRoot);
    }

    /// @notice Dismisses a failed challenge and opens a fresh safety window.
    function dismissChallenge(bytes32 certificateId, bytes32 resolutionEvidenceRoot, uint64 renewedChallengePeriod)
        external
        onlyRole(RESOLVER_ROLE)
    {
        Certificate storage certificate = certificates[certificateId];
        if (certificate.state != CertificateState.Challenged) revert InvalidState();
        if (resolutionEvidenceRoot == bytes32(0) || renewedChallengePeriod < MINIMUM_CHALLENGE_PERIOD) {
            revert InvalidInput();
        }
        certificate.resolutionEvidenceRoot = resolutionEvidenceRoot;
        certificate.challengeEndsAt = uint64(block.timestamp) + renewedChallengePeriod;
        certificate.state = CertificateState.Pending;
        emit ChallengeDismissed(certificateId, resolutionEvidenceRoot, certificate.challengeEndsAt);
    }

    function resolveChallenge(bytes32 certificateId, bool reject, bytes32 resolutionEvidenceRoot)
        external
        onlyRole(RESOLVER_ROLE)
    {
        Certificate storage certificate = certificates[certificateId];
        if (certificate.state != CertificateState.Challenged) revert InvalidState();
        if (resolutionEvidenceRoot == bytes32(0)) revert InvalidInput();
        certificate.resolutionEvidenceRoot = resolutionEvidenceRoot;
        if (reject) {
            certificate.state = CertificateState.Rejected;
            emit Rejected(certificateId, resolutionEvidenceRoot);
        } else {
            certificate.state = CertificateState.Quarantined;
            emit Quarantined(certificateId, resolutionEvidenceRoot);
        }
    }

    function finalize(bytes32 certificateId) external {
        Certificate storage certificate = certificates[certificateId];
        if (certificate.state == CertificateState.Unset) revert MissingCertificate();
        if (certificate.state != CertificateState.Pending) revert InvalidState();
        if (block.timestamp <= certificate.challengeEndsAt) revert ChallengeWindowOpen();
        certificate.state = CertificateState.Final;
        emit Finalized(certificateId);
    }

    /// @notice Emergency fail-closed path for newly discovered checker compromise.
    function quarantine(bytes32 certificateId, bytes32 evidenceRoot) external onlyRole(RESOLVER_ROLE) {
        Certificate storage certificate = certificates[certificateId];
        if (certificate.state == CertificateState.Unset) revert MissingCertificate();
        if (evidenceRoot == bytes32(0)) revert InvalidInput();
        certificate.resolutionEvidenceRoot = evidenceRoot;
        certificate.state = CertificateState.Quarantined;
        emit Quarantined(certificateId, evidenceRoot);
    }

    function isFinalFor(bytes32 certificateId, bytes32 claimId, bytes32 artifactRoot, bytes32 policyId)
        external
        view
        returns (bool)
    {
        Certificate storage certificate = certificates[certificateId];
        return certificate.state == CertificateState.Final && certificate.claimId == claimId
            && certificate.artifactRoot == artifactRoot && certificate.policyId == policyId;
    }
}
