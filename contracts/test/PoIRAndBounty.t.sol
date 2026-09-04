// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {PoIRCertificateRegistry} from "../src/PoIRCertificateRegistry.sol";
import {BountyEscrow, IPoIRCertificateRegistry} from "../src/BountyEscrow.sol";
import {MockUSDC} from "./mocks/MockUSDC.sol";

contract PoIRAndBountyTest is Test {
    MockUSDC internal asset;
    PoIRCertificateRegistry internal registry;
    BountyEscrow internal bountyEscrow;

    address internal sponsor = makeAddr("sponsor");
    address internal solver = makeAddr("solver");

    bytes32 internal constant CLAIM_ID = keccak256("claim");
    bytes32 internal constant PROOF_ID = keccak256("proof");
    bytes32 internal constant ARTIFACT_ROOT = keccak256("artifact");
    bytes32 internal constant POLICY_ID = keccak256("gold-policy");
    bytes32 internal constant OBSERVATION_ROOT = keccak256("observations");
    bytes32 internal constant CERTIFICATE_ID = keccak256("certificate");

    function setUp() public {
        asset = new MockUSDC();
        registry = new PoIRCertificateRegistry(address(this));
        bountyEscrow = new BountyEscrow(
            IPoIRCertificateRegistry(address(registry)),
            address(this)
        );
        asset.mint(sponsor, 1_000_000e6);
    }

    function _submitAndFinalizeCertificate() internal {
        registry.submit(
            CERTIFICATE_ID,
            CLAIM_ID,
            PROOF_ID,
            ARTIFACT_ROOT,
            POLICY_ID,
            OBSERVATION_ROOT,
            bytes32(0),
            1 hours
        );
        vm.warp(block.timestamp + 1 hours + 1);
        registry.finalize(CERTIFICATE_ID);
    }

    function testCertificateCannotFinalizeBeforeChallengeWindow() public {
        registry.submit(
            CERTIFICATE_ID,
            CLAIM_ID,
            PROOF_ID,
            ARTIFACT_ROOT,
            POLICY_ID,
            OBSERVATION_ROOT,
            bytes32(0),
            1 hours
        );

        vm.expectRevert(PoIRCertificateRegistry.ChallengeWindowOpen.selector);
        registry.finalize(CERTIFICATE_ID);
    }

    function testChallengePreventsFinalization() public {
        registry.submit(
            CERTIFICATE_ID,
            CLAIM_ID,
            PROOF_ID,
            ARTIFACT_ROOT,
            POLICY_ID,
            OBSERVATION_ROOT,
            bytes32(0),
            1 hours
        );
        registry.challenge(CERTIFICATE_ID, keccak256("counterevidence"));

        vm.warp(block.timestamp + 2 hours);
        vm.expectRevert(PoIRCertificateRegistry.InvalidState.selector);
        registry.finalize(CERTIFICATE_ID);
    }

    function testBountyRequiresCommitRevealAndFinalMatchingCertificate() public {
        _submitAndFinalizeCertificate();

        bytes32 bountyId = keccak256("bounty");
        bytes32 salt = keccak256("secret");
        uint64 deadline = uint64(block.timestamp + 7 days);

        vm.startPrank(sponsor);
        asset.approve(address(bountyEscrow), 500e6);
        bountyEscrow.create(
            bountyId,
            IERC20(address(asset)),
            500e6,
            CLAIM_ID,
            POLICY_ID,
            deadline,
            1 days
        );
        vm.stopPrank();

        bytes32 commitment = keccak256(
            abi.encode(bountyId, ARTIFACT_ROOT, salt, solver)
        );
        vm.prank(solver);
        bountyEscrow.commitSolution(bountyId, commitment);

        bountyEscrow.acceptCertifiedSolution(
            bountyId,
            solver,
            ARTIFACT_ROOT,
            salt,
            CERTIFICATE_ID
        );

        vm.expectRevert(BountyEscrow.ReleaseDelayOpen.selector);
        bountyEscrow.release(bountyId);

        vm.warp(block.timestamp + 1 days);
        bountyEscrow.release(bountyId);
        assertEq(asset.balanceOf(solver), 500e6);
    }

    function testQuarantineAfterAcceptanceBlocksRelease() public {
        _submitAndFinalizeCertificate();

        bytes32 bountyId = keccak256("bounty-quarantine");
        bytes32 salt = keccak256("secret-2");
        vm.startPrank(sponsor);
        asset.approve(address(bountyEscrow), 100e6);
        bountyEscrow.create(
            bountyId,
            IERC20(address(asset)),
            100e6,
            CLAIM_ID,
            POLICY_ID,
            uint64(block.timestamp + 7 days),
            1 hours
        );
        vm.stopPrank();

        vm.prank(solver);
        bountyEscrow.commitSolution(
            bountyId,
            keccak256(abi.encode(bountyId, ARTIFACT_ROOT, salt, solver))
        );
        bountyEscrow.acceptCertifiedSolution(
            bountyId,
            solver,
            ARTIFACT_ROOT,
            salt,
            CERTIFICATE_ID
        );

        registry.quarantine(CERTIFICATE_ID, keccak256("checker-compromise"));
        vm.warp(block.timestamp + 1 hours);
        vm.expectRevert(BountyEscrow.CertificateNotFinal.selector);
        bountyEscrow.release(bountyId);
    }
}
