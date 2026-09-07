// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {ResearchCommitmentRegistry} from "../src/ResearchCommitmentRegistry.sol";

contract ResearchCommitmentRegistryTest is Test {
    ResearchCommitmentRegistry internal registry;

    function setUp() public {
        registry = new ResearchCommitmentRegistry(address(this));
    }

    function _commitment(bytes32 claimId, bytes32 parentId)
        internal
        pure
        returns (ResearchCommitmentRegistry.Commitment memory)
    {
        return ResearchCommitmentRegistry.Commitment({
            researcherId: keccak256("researcher"),
            claimId: claimId,
            artifactRoot: keccak256("artifact"),
            protocolPolicyId: keccak256("XLMP/1-policy"),
            committeeAssignmentRoot: keccak256("committee"),
            rightsManifestRoot: keccak256("rights"),
            contributionSplitRoot: keccak256("contributions"),
            parentCommitmentId: parentId,
            createdAt: 0,
            supersededBy: bytes32(0)
        });
    }

    function testCommitsAllOnChainBoundaryRoots() public {
        ResearchCommitmentRegistry.Commitment memory proposed = _commitment(keccak256("claim"), bytes32(0));
        bytes32 commitmentId = registry.deriveCommitmentId(proposed);
        registry.commit(commitmentId, proposed);

        ResearchCommitmentRegistry.Commitment memory stored = registry.getCommitment(commitmentId);
        assertEq(stored.researcherId, proposed.researcherId);
        assertEq(stored.claimId, proposed.claimId);
        assertEq(stored.artifactRoot, proposed.artifactRoot);
        assertEq(stored.protocolPolicyId, proposed.protocolPolicyId);
        assertEq(stored.committeeAssignmentRoot, proposed.committeeAssignmentRoot);
        assertEq(stored.rightsManifestRoot, proposed.rightsManifestRoot);
        assertEq(stored.contributionSplitRoot, proposed.contributionSplitRoot);
    }

    function testContentIdentityRejectsRightsSubstitution() public {
        ResearchCommitmentRegistry.Commitment memory proposed = _commitment(keccak256("claim"), bytes32(0));
        bytes32 commitmentId = registry.deriveCommitmentId(proposed);
        proposed.rightsManifestRoot = keccak256("substituted-rights");

        vm.expectRevert(ResearchCommitmentRegistry.InvalidInput.selector);
        registry.commit(commitmentId, proposed);
    }

    function testCorrectionPreservesOriginAndCreatesExplicitSupersession() public {
        ResearchCommitmentRegistry.Commitment memory original = _commitment(keccak256("claim-v1"), bytes32(0));
        bytes32 originalId = registry.deriveCommitmentId(original);
        registry.commit(originalId, original);

        ResearchCommitmentRegistry.Commitment memory correction = _commitment(keccak256("claim-v2"), originalId);
        bytes32 correctionId = registry.deriveCommitmentId(correction);
        registry.commit(correctionId, correction);
        registry.supersede(originalId, correctionId);

        assertEq(registry.getCommitment(originalId).supersededBy, correctionId);
        assertEq(registry.getCommitment(correctionId).parentCommitmentId, originalId);
        assertEq(registry.getCommitment(correctionId).researcherId, original.researcherId);
    }
}
