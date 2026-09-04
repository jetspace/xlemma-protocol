// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ResearchVault} from "../src/ResearchVault.sol";
import {ResearcherCredit} from "../src/ResearcherCredit.sol";
import {MockUSDC} from "./mocks/MockUSDC.sol";

contract ResearchVaultTest is Test {
    MockUSDC internal asset;
    ResearchVault internal vault;
    ResearcherCredit internal credit;

    address internal researcher = makeAddr("researcher");
    address internal verifier = makeAddr("verifier");
    address internal outsider = makeAddr("outsider");

    function setUp() public {
        asset = new MockUSDC();
        vault = new ResearchVault(
            IERC20(address(asset)),
            6,
            researcher,
            address(this),
            "Research Credit",
            "RCR"
        );
        credit = vault.credit();
        asset.mint(researcher, 1_000_000e6);
    }

    function _deposit(uint256 amount) internal {
        vm.startPrank(researcher);
        asset.approve(address(vault), amount);
        vault.deposit(amount, researcher);
        vm.stopPrank();
    }

    function testDepositMintsOnlyMatchingBackedCredits() public {
        _deposit(1_000e6);

        assertEq(asset.balanceOf(address(vault)), 1_000e6);
        assertEq(credit.balanceOf(researcher), 1_000e6);
        assertEq(credit.totalSupply(), 1_000e6);
        assertTrue(vault.isSolvent());
    }

    function testAuthorizationSettlesActualAndRefundsUnused() public {
        _deposit(1_000e6);
        bytes32 authorizationId = keccak256("job-1");

        vm.startPrank(researcher);
        credit.approve(address(vault), 40e6);
        vault.authorize(
            authorizationId,
            verifier,
            40e6,
            uint64(block.timestamp + 1 days)
        );
        vm.stopPrank();

        vault.settle(authorizationId, 25e6);

        assertEq(asset.balanceOf(verifier), 25e6);
        assertEq(asset.balanceOf(address(vault)), 975e6);
        assertEq(credit.balanceOf(researcher), 975e6);
        assertEq(credit.totalSupply(), 975e6);
        assertTrue(vault.isSolvent());
    }

    function testAuthorizationCannotSettleTwice() public {
        _deposit(100e6);
        bytes32 authorizationId = keccak256("job-once");

        vm.startPrank(researcher);
        credit.approve(address(vault), 10e6);
        vault.authorize(
            authorizationId,
            verifier,
            10e6,
            uint64(block.timestamp + 1 days)
        );
        vm.stopPrank();

        vault.settle(authorizationId, 10e6);
        vm.expectRevert(ResearchVault.AuthorizationClosed.selector);
        vault.settle(authorizationId, 10e6);
    }

    function testSettlementCannotExceedMaximum() public {
        _deposit(100e6);
        bytes32 authorizationId = keccak256("job-max");

        vm.startPrank(researcher);
        credit.approve(address(vault), 10e6);
        vault.authorize(
            authorizationId,
            verifier,
            10e6,
            uint64(block.timestamp + 1 days)
        );
        vm.stopPrank();

        vm.expectRevert(ResearchVault.ExceedsMaximum.selector);
        vault.settle(authorizationId, 11e6);
    }

    function testExpiredAuthorizationReturnsAllCredits() public {
        _deposit(100e6);
        bytes32 authorizationId = keccak256("job-expired");
        uint64 expiresAt = uint64(block.timestamp + 1 hours);

        vm.startPrank(researcher);
        credit.approve(address(vault), 20e6);
        vault.authorize(authorizationId, verifier, 20e6, expiresAt);
        vm.stopPrank();

        vm.warp(uint256(expiresAt) + 1);
        vault.cancelExpired(authorizationId);

        assertEq(credit.balanceOf(researcher), 100e6);
        assertEq(credit.balanceOf(address(vault)), 0);
        assertTrue(vault.isSolvent());
    }

    function testResearchCreditsCannotMovePeerToPeer() public {
        _deposit(100e6);

        vm.startPrank(researcher);
        vm.expectRevert(ResearcherCredit.RestrictedTransfer.selector);
        credit.transfer(outsider, 1e6);
        vm.stopPrank();
    }

    function testSettledExternalRevenueCanCompoundIntoCredits() public {
        asset.mint(address(this), 60e6);
        asset.approve(address(vault), 60e6);

        vault.compoundRevenue(60e6, researcher);

        assertEq(asset.balanceOf(address(vault)), 60e6);
        assertEq(credit.balanceOf(researcher), 60e6);
        assertTrue(vault.isSolvent());
    }

    function testRedeemBurnsCreditsAndReleasesMatchingBacking() public {
        _deposit(100e6);

        vm.prank(researcher);
        vault.redeem(30e6, researcher);

        assertEq(asset.balanceOf(researcher), 999_930e6);
        assertEq(asset.balanceOf(address(vault)), 70e6);
        assertEq(credit.totalSupply(), 70e6);
        assertTrue(vault.isSolvent());
    }

    function testFuzzSettlementPreservesSolvency(
        uint96 depositAmount,
        uint96 maximum,
        uint96 actual
    ) public {
        uint256 deposit = bound(uint256(depositAmount), 1, 1_000_000e6);
        uint256 maxAuthorization = bound(uint256(maximum), 1, deposit);
        uint256 actualCharge = bound(uint256(actual), 0, maxAuthorization);
        _deposit(deposit);

        bytes32 authorizationId = keccak256(
            abi.encode(deposit, maxAuthorization, actualCharge)
        );
        vm.startPrank(researcher);
        credit.approve(address(vault), maxAuthorization);
        vault.authorize(
            authorizationId,
            verifier,
            maxAuthorization,
            uint64(block.timestamp + 1 days)
        );
        vm.stopPrank();

        vault.settle(authorizationId, actualCharge);

        assertGe(asset.balanceOf(address(vault)), credit.totalSupply());
        assertEq(asset.balanceOf(verifier), actualCharge);
    }
}
