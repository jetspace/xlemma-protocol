// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ResearchVault} from "../src/ResearchVault.sol";
import {ResearcherCredit} from "../src/ResearcherCredit.sol";
import {RevenueRouter, IResearchVault} from "../src/RevenueRouter.sol";
import {MockUSDC} from "./mocks/MockUSDC.sol";

contract RevenueRouterTest is Test {
    MockUSDC internal asset;
    ResearchVault internal vault;
    RevenueRouter internal router;
    ResearcherCredit internal credit;

    address internal researcher = makeAddr("researcher");
    address internal payer = makeAddr("customer");
    address internal cashRecipient = makeAddr("cash-recipient");

    function setUp() public {
        asset = new MockUSDC();
        vault = new ResearchVault(IERC20(address(asset)), 6, researcher, address(this), "Research Credit", "RCR");
        credit = vault.credit();
        router = new RevenueRouter();
        vault.grantRole(vault.REVENUE_ROUTER_ROLE(), address(router));
        asset.mint(payer, 1_000_000e6);
    }

    function testRoutesCashAndCompoundsExternalRevenue() public {
        RevenueRouter.CashRecipient[] memory cash = new RevenueRouter.CashRecipient[](1);
        cash[0] = RevenueRouter.CashRecipient({account: cashRecipient, shareBps: 4_000});

        RevenueRouter.CompoundRecipient[] memory compound = new RevenueRouter.CompoundRecipient[](1);
        compound[0] = RevenueRouter.CompoundRecipient({
            vault: IResearchVault(address(vault)), beneficiary: researcher, shareBps: 6_000
        });

        vm.startPrank(payer);
        asset.approve(address(router), 1_000e6);
        router.routeWithCompounding(keccak256("revenue-1"), IERC20(address(asset)), 1_000e6, cash, compound);
        vm.stopPrank();

        assertEq(asset.balanceOf(cashRecipient), 400e6);
        assertEq(asset.balanceOf(address(vault)), 600e6);
        assertEq(credit.balanceOf(researcher), 600e6);
        assertEq(credit.totalSupply(), 600e6);
        assertTrue(vault.isSolvent());
    }

    function testRevenueEventCannotReplay() public {
        bytes32 eventId = keccak256("revenue-once");
        RevenueRouter.CashRecipient[] memory cash = new RevenueRouter.CashRecipient[](1);
        cash[0] = RevenueRouter.CashRecipient({account: cashRecipient, shareBps: 10_000});

        vm.startPrank(payer);
        asset.approve(address(router), 200e6);
        router.route(eventId, IERC20(address(asset)), 100e6, cash);
        vm.expectRevert(RevenueRouter.InvalidEvent.selector);
        router.route(eventId, IERC20(address(asset)), 100e6, cash);
        vm.stopPrank();
    }

    function testAnotherPayerCannotSquatARevenueEventNamespace() public {
        bytes32 eventId = keccak256("payer-local-event");
        address otherPayer = makeAddr("other-customer");
        asset.mint(otherPayer, 100e6);
        RevenueRouter.CashRecipient[] memory cash = new RevenueRouter.CashRecipient[](1);
        cash[0] = RevenueRouter.CashRecipient({account: cashRecipient, shareBps: 10_000});

        vm.startPrank(otherPayer);
        asset.approve(address(router), 50e6);
        router.route(eventId, IERC20(address(asset)), 50e6, cash);
        vm.stopPrank();

        vm.startPrank(payer);
        asset.approve(address(router), 50e6);
        router.route(eventId, IERC20(address(asset)), 50e6, cash);
        vm.stopPrank();
        assertEq(asset.balanceOf(cashRecipient), 100e6);
    }

    function testSharesMustConserveTenThousandBasisPoints() public {
        RevenueRouter.CashRecipient[] memory cash = new RevenueRouter.CashRecipient[](1);
        cash[0] = RevenueRouter.CashRecipient({account: cashRecipient, shareBps: 9_999});

        vm.startPrank(payer);
        asset.approve(address(router), 100e6);
        vm.expectRevert(RevenueRouter.InvalidShares.selector);
        router.route(keccak256("bad-shares"), IERC20(address(asset)), 100e6, cash);
        vm.stopPrank();
    }
}
