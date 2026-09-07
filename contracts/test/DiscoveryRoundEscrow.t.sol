// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {DiscoveryRoundEscrow, IDiscoveryEvidenceRegistry} from "../src/DiscoveryRoundEscrow.sol";
import {MockUSDC} from "./mocks/MockUSDC.sol";

contract DiscoveryEvidenceFixture is IDiscoveryEvidenceRegistry {
    bool public valid = true;

    function setValid(bool value) external {
        valid = value;
    }

    function isFinalFor(bytes32 certificate, bytes32 claim, bytes32 artifact, bytes32 policy)
        external
        view
        returns (bool)
    {
        return valid && certificate == bytes32(uint256(1)) && claim == bytes32(uint256(2))
            && artifact == bytes32(uint256(3)) && policy == bytes32(uint256(4));
    }
}

contract DiscoveryRoundEscrowTest is Test {
    MockUSDC token;
    DiscoveryEvidenceFixture evidence;
    DiscoveryRoundEscrow escrow;
    address donor = address(10);
    address researcher = address(11);
    address reviewerA = address(12);
    address reviewerB = address(13);
    bytes32 constant ROUND = bytes32(uint256(100));
    bytes32 constant PLAN = bytes32(uint256(101));
    uint64 opens;
    uint64 appeals;
    uint64 expires;

    function setUp() public {
        token = new MockUSDC();
        evidence = new DiscoveryEvidenceFixture();
        escrow = new DiscoveryRoundEscrow(token, evidence, address(this));
        escrow.grantRole(escrow.PLANNER_ROLE(), address(this));
        escrow.grantRole(escrow.WATCHER_ROLE(), address(this));
        escrow.setOperatorCluster(address(this), bytes32(uint256(1)));
        _reviewer(reviewerA, 2);
        _reviewer(reviewerB, 3);
        opens = uint64(block.timestamp + 1 days);
        appeals = opens + 2 days;
        expires = appeals + 3 days;
        uint256[7] memory solver;
        uint256[7] memory review;
        solver[0] = 300e6;
        review[0] = 20e6;
        escrow.createRound(ROUND, bytes32(uint256(7)), opens, opens + 1 days, appeals, expires, 10000, solver, review);
        token.mint(donor, 500e6);
        vm.startPrank(donor);
        token.approve(address(escrow), 500e6);
        escrow.fund(ROUND, 0, 500e6, bytes32(uint256(8)));
        vm.stopPrank();
    }

    function _reviewer(address who, uint256 cluster) internal {
        escrow.setOperatorCluster(who, bytes32(cluster));
        escrow.grantRole(escrow.REVIEWER_ROLE(), who);
        escrow.grantRole(escrow.RESOLVER_ROLE(), who);
    }

    function _items(uint256 amount, bool review) internal view returns (DiscoveryRoundEscrow.Item[] memory items) {
        items = new DiscoveryRoundEscrow.Item[](1);
        items[0] = DiscoveryRoundEscrow.Item(
            0,
            review,
            researcher,
            amount,
            bytes32(uint256(1)),
            bytes32(uint256(2)),
            bytes32(uint256(3)),
            bytes32(uint256(4)),
            bytes32(uint256(5))
        );
    }

    function _propose(uint256 amount, bool review) internal {
        vm.warp(appeals);
        escrow.proposePlan(ROUND, PLAN, _items(amount, review));
    }

    function _approve() internal {
        bytes32 commitment = escrow.planCommitment(PLAN);
        vm.prank(reviewerA);
        escrow.approvePlan(PLAN, commitment);
        vm.prank(reviewerB);
        escrow.approvePlan(PLAN, commitment);
        vm.warp(block.timestamp + 1 hours);
    }

    function testPaysOnceAndRefundsUnusedCategoryFunds() public {
        _propose(300e6, false);
        _approve();
        escrow.executePlan(ROUND);
        assertEq(token.balanceOf(researcher), 300e6);
        vm.expectRevert();
        escrow.executePlan(ROUND);
        vm.prank(donor);
        escrow.refund(ROUND, 0);
        assertEq(token.balanceOf(donor), 200e6);
        vm.prank(donor);
        vm.expectRevert();
        escrow.refund(ROUND, 0);
        assertEq(token.balanceOf(address(escrow)), 0);
    }

    function testRejectedEvidenceStillPermitsCompletedReviewPayment() public {
        evidence.setValid(false);
        _propose(10e6, true);
        _approve();
        escrow.executePlan(ROUND);
        assertEq(token.balanceOf(researcher), 10e6);
    }

    function testRevokedEvidenceBlocksEntireAtomicRewardBatch() public {
        _propose(300e6, false);
        _approve();
        evidence.setValid(false);
        vm.expectRevert(DiscoveryRoundEscrow.EvidenceNotFinal.selector);
        escrow.executePlan(ROUND);
        assertEq(token.balanceOf(researcher), 0);
        assertFalse(escrow.round(ROUND).paid);
    }

    function testRoleOrClusterRevocationInvalidatesApproval() public {
        _propose(300e6, false);
        _approve();
        escrow.setOperatorCluster(reviewerA, bytes32(uint256(9)));
        vm.expectRevert();
        escrow.executePlan(ROUND);
    }

    function testSameControlClusterCannotCountTwice() public {
        escrow.setOperatorCluster(reviewerB, bytes32(uint256(2)));
        _propose(300e6, false);
        bytes32 c = escrow.planCommitment(PLAN);
        vm.prank(reviewerA);
        escrow.approvePlan(PLAN, c);
        vm.prank(reviewerB);
        vm.expectRevert();
        escrow.approvePlan(PLAN, c);
    }

    function testCapsCannotBorrowFromUnusedFundsOrCategories() public {
        vm.warp(appeals);
        vm.expectRevert(DiscoveryRoundEscrow.InsufficientFunding.selector);
        escrow.proposePlan(ROUND, PLAN, _items(301e6, false));
        vm.expectRevert(DiscoveryRoundEscrow.InsufficientFunding.selector);
        escrow.proposePlan(ROUND, PLAN, _items(21e6, true));
        DiscoveryRoundEscrow.Item[] memory items = _items(1, false);
        items[0].category = 1;
        vm.expectRevert(DiscoveryRoundEscrow.InsufficientFunding.selector);
        escrow.proposePlan(ROUND, PLAN, items);
    }

    function testHoldNeedsIndependentResolutionAndNewDelay() public {
        _propose(300e6, false);
        _approve();
        bytes32 holdRoot = bytes32(uint256(88));
        escrow.hold(ROUND, holdRoot);
        vm.expectRevert();
        escrow.executePlan(ROUND);
        vm.prank(reviewerA);
        escrow.resolveHold(ROUND, bytes32(uint256(89)));
        vm.expectRevert();
        escrow.executePlan(ROUND);
        vm.prank(reviewerB);
        escrow.resolveHold(ROUND, bytes32(uint256(89)));
        vm.expectRevert();
        escrow.executePlan(ROUND);
        vm.expectRevert();
        escrow.hold(ROUND, holdRoot);
        vm.warp(block.timestamp + 1 hours);
        escrow.executePlan(ROUND);
    }

    function testExpiryReturnsUnspentFundingWithoutApprovingResearch() public {
        _propose(300e6, false);
        vm.warp(expires);
        escrow.expire(ROUND);
        vm.prank(donor);
        escrow.refund(ROUND, 0);
        assertEq(token.balanceOf(donor), 500e6);
        vm.expectRevert();
        escrow.executePlan(ROUND);
    }

    function testNoEarlyExecutionOrFundingAfterOpening() public {
        vm.warp(opens);
        vm.prank(donor);
        vm.expectRevert();
        escrow.fund(ROUND, 0, 1, bytes32(uint256(8)));
        _propose(300e6, false);
        vm.expectRevert();
        escrow.executePlan(ROUND);
    }

    function testFuzzConservesFundedUnits(uint96 amount) public {
        uint256 payout = bound(uint256(amount), 1, 300e6);
        _propose(payout, false);
        _approve();
        escrow.executePlan(ROUND);
        vm.prank(donor);
        escrow.refund(ROUND, 0);
        assertEq(token.balanceOf(researcher) + token.balanceOf(donor) + token.balanceOf(address(escrow)), 500e6);
    }

    function testRepeatedWorkInvoiceCannotConsumeAnotherRound() public {
        _propose(10e6, true);
        _approve();
        escrow.executePlan(ROUND);
        uint64 nextOpen = uint64(block.timestamp + 1 days);
        uint256[7] memory solver;
        uint256[7] memory review;
        solver[0] = 300e6;
        review[0] = 20e6;
        bytes32 otherRound = bytes32(uint256(200));
        bytes32 otherPlan = bytes32(uint256(201));
        escrow.createRound(
            otherRound,
            bytes32(uint256(7)),
            nextOpen,
            nextOpen + 1 days,
            nextOpen + 2 days,
            nextOpen + 4 days,
            10000,
            solver,
            review
        );
        token.mint(donor, 20e6);
        vm.startPrank(donor);
        token.approve(address(escrow), 20e6);
        escrow.fund(otherRound, 0, 20e6, bytes32(uint256(8)));
        vm.stopPrank();
        vm.warp(nextOpen + 2 days);
        escrow.proposePlan(otherRound, otherPlan, _items(10e6, true));
        bytes32 commitment = escrow.planCommitment(otherPlan);
        vm.prank(reviewerA);
        escrow.approvePlan(otherPlan, commitment);
        vm.prank(reviewerB);
        escrow.approvePlan(otherPlan, commitment);
        vm.warp(block.timestamp + 1 hours);
        vm.expectRevert(DiscoveryRoundEscrow.WorkAlreadyPaid.selector);
        escrow.executePlan(otherRound);
        assertEq(token.balanceOf(researcher), 10e6);
        assertFalse(escrow.round(otherRound).paid);
    }
}
