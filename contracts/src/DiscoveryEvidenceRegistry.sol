// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";

/// @notice Publishes exact protocol certificate digests for discovery escrow.
/// @dev Qualified relays reproduce the authenticated certificate and complete
///      observation history offchain. Relay signatures are publication authority,
///      never a substitute for Lean checking or experimental evidence. Any watcher
///      can permanently quarantine a digest; a majority cannot remove that hold.
contract DiscoveryEvidenceRegistry is AccessControl {
    bytes32 public constant RELAY_ROLE = keccak256("RELAY_ROLE");
    bytes32 public constant WATCHER_ROLE = keccak256("WATCHER_ROLE");
    mapping(address => bytes32) public operatorCluster;

    struct Evidence {
        bytes32 claim;
        bytes32 artifact;
        bytes32 policy;
        bytes32 observationRoot;
        bytes32 producersRoot;
        uint64 challengeEndsAt;
        uint64 publishedAt;
        bool quarantined;
        address[] relays;
        bytes32[] relayClusters;
    }
    mapping(bytes32 => Evidence) private records;
    // Keep quarantine even if it arrives before publication.
    mapping(bytes32 => bytes32) public quarantineRoot;
    uint64 public immutable publicationDelay;

    event Published(
        bytes32 indexed certificate,
        bytes32 indexed claim,
        bytes32 artifact,
        bytes32 policy,
        bytes32 observationRoot,
        bytes32 producersRoot,
        uint64 challengeEndsAt,
        address relay
    );
    event Quarantined(bytes32 indexed certificate, bytes32 evidenceRoot);
    error Invalid();

    constructor(address administrator, uint64 delay) {
        if (administrator == address(0) || delay < 1 hours) revert Invalid();
        publicationDelay = delay;
        _grantRole(DEFAULT_ADMIN_ROLE, administrator);
    }

    function setOperatorCluster(address operator, bytes32 cluster) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (operator == address(0) || cluster == bytes32(0)) revert Invalid();
        operatorCluster[operator] = cluster;
    }

    function attest(
        bytes32 certificate,
        bytes32 claim,
        bytes32 artifact,
        bytes32 policy,
        bytes32 observationRoot,
        bytes32[] calldata producerClusters,
        uint64 challengeEndsAt
    ) external onlyRole(RELAY_ROLE) {
        bytes32 cluster = operatorCluster[msg.sender];
        if (
            certificate == bytes32(0) || claim == bytes32(0) || artifact == bytes32(0) || policy == bytes32(0)
                || observationRoot == bytes32(0) || cluster == bytes32(0) || producerClusters.length == 0
                || producerClusters.length > 254 || challengeEndsAt == 0 || quarantineRoot[certificate] != bytes32(0)
        ) revert Invalid();
        bytes32 prior = bytes32(0);
        for (uint256 i = 0; i < producerClusters.length; ++i) {
            if (producerClusters[i] <= prior || producerClusters[i] == cluster) revert Invalid();
            prior = producerClusters[i];
        }
        bytes32 producersRoot = keccak256(abi.encode(producerClusters));
        Evidence storage e = records[certificate];
        if (e.claim == bytes32(0)) {
            e.claim = claim;
            e.artifact = artifact;
            e.policy = policy;
            e.observationRoot = observationRoot;
            e.producersRoot = producersRoot;
            e.challengeEndsAt = challengeEndsAt;
            e.publishedAt = uint64(block.timestamp);
        } else if (
            e.claim != claim || e.artifact != artifact || e.policy != policy || e.observationRoot != observationRoot
                || e.producersRoot != producersRoot || e.challengeEndsAt != challengeEndsAt || e.quarantined
        ) {
            revert Invalid();
        }
        if (e.relays.length >= 8) revert Invalid();
        for (uint256 i = 0; i < e.relays.length; ++i) {
            if (e.relays[i] == msg.sender || e.relayClusters[i] == cluster) revert Invalid();
        }
        e.relays.push(msg.sender);
        e.relayClusters.push(cluster);
        // Every new publication must remain observable before it can enable payout.
        e.publishedAt = uint64(block.timestamp);
        emit Published(
            certificate, claim, artifact, policy, observationRoot, producersRoot, challengeEndsAt, msg.sender
        );
    }

    function quarantine(bytes32 certificate, bytes32 evidenceRoot) external onlyRole(WATCHER_ROLE) {
        if (certificate == bytes32(0) || evidenceRoot == bytes32(0) || quarantineRoot[certificate] != bytes32(0)) {
            revert Invalid();
        }
        quarantineRoot[certificate] = evidenceRoot;
        records[certificate].quarantined = true;
        emit Quarantined(certificate, evidenceRoot);
    }

    function isFinalFor(bytes32 certificate, bytes32 claim, bytes32 artifact, bytes32 policy)
        external
        view
        returns (bool)
    {
        Evidence storage e = records[certificate];
        if (
            e.claim == bytes32(0) || e.claim != claim || e.artifact != artifact || e.policy != policy || e.quarantined
                || block.timestamp < e.challengeEndsAt || block.timestamp < uint256(e.publishedAt) + publicationDelay
        ) return false;
        uint256 independent = 0;
        for (uint256 i = 0; i < e.relays.length; ++i) {
            if (hasRole(RELAY_ROLE, e.relays[i]) && operatorCluster[e.relays[i]] == e.relayClusters[i]) ++independent;
        }
        return independent >= 2;
    }
}
