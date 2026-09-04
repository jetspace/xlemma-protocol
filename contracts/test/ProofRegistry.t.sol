// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {ProofRegistry} from "../src/ProofRegistry.sol";

contract ProofRegistryTest is Test {
    ProofRegistry internal registry;

    function setUp() public {
        registry = new ProofRegistry(address(this));
    }

    function _record(bytes32 claimId, bytes32 artifactRoot)
        internal
        pure
        returns (ProofRegistry.Record memory)
    {
        return ProofRegistry.Record({
            theoryId: keccak256("theory"),
            claimId: claimId,
            proofId: keccak256("proof"),
            artifactRoot: artifactRoot,
            policyId: keccak256("policy"),
            certificateRoot: bytes32(0),
            parentRecordId: bytes32(0),
            createdAt: 0,
            challengeEndsAt: 0,
            status: ProofRegistry.FormalStatus.Committed
        });
    }

    function testRecordIsAppendOnlyAndCertificateAttachesOnce() public {
        bytes32 recordId = keccak256("record-1");
        registry.commitRecord(
            recordId,
            _record(keccak256("claim-1"), keccak256("artifact-1"))
        );
        registry.attachCertificate(
            recordId,
            keccak256("certificate-root"),
            ProofRegistry.FormalStatus.Certified,
            uint64(block.timestamp + 1 days)
        );

        vm.expectRevert(ProofRegistry.InvalidTransition.selector);
        registry.attachCertificate(
            recordId,
            keccak256("other-certificate"),
            ProofRegistry.FormalStatus.Certified,
            uint64(block.timestamp + 1 days)
        );
    }

    function testCorrectionCreatesSupersessionEdge() public {
        bytes32 oldId = keccak256("old-record");
        bytes32 newId = keccak256("new-record");
        registry.commitRecord(
            oldId,
            _record(keccak256("claim-old"), keccak256("artifact-old"))
        );
        registry.commitRecord(
            newId,
            _record(keccak256("claim-new"), keccak256("artifact-new"))
        );

        registry.supersede(oldId, newId);

        ProofRegistry.Record memory oldRecord = registry.getRecord(oldId);
        ProofRegistry.Record memory newRecord = registry.getRecord(newId);
        assertEq(oldRecord.parentRecordId, bytes32(0));
        assertEq(newRecord.parentRecordId, oldId);
        assertEq(
            uint256(oldRecord.status),
            uint256(ProofRegistry.FormalStatus.Superseded)
        );
    }

    function testQuarantineDoesNotDeleteRecord() public {
        bytes32 recordId = keccak256("quarantine-record");
        bytes32 claimId = keccak256("claim-q");
        registry.commitRecord(
            recordId,
            _record(claimId, keccak256("artifact-q"))
        );
        registry.quarantine(recordId, keccak256("evidence"));

        ProofRegistry.Record memory record = registry.getRecord(recordId);
        assertEq(record.claimId, claimId);
        assertEq(
            uint256(record.status),
            uint256(ProofRegistry.FormalStatus.Quarantined)
        );
    }
}
