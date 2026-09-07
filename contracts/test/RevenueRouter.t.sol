// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ResearchVault} from "../src/ResearchVault.sol";
import {ResearcherCredit} from "../src/ResearcherCredit.sol";
import {RevenueRouter, IResearchVault} from "../src/RevenueRouter.sol";
import {MockUSDC} from "./mocks/MockUSDC.sol";

contract NoopVault is IResearchVault {
    IERC20 public immutable override settlementAsset;

    constructor(IERC20 asset) {
        settlementAsset = asset;
    }
    function compoundRevenue(uint256, address) external {
        // Deliberately returns without consuming an allocation.
    }
}

    contract ReenteringVault is IResearchVault {
        IERC20 public immutable override settlementAsset;
        RevenueRouter public immutable router;
        bool public reentryBlocked;

        constructor(IERC20 asset, RevenueRouter router_) {
            settlementAsset = asset;
            router = router_;
        }

        function compoundRevenue(uint256 amount, address beneficiary) external {
            RevenueRouter.CashRecipient[] memory recipients = new RevenueRouter.CashRecipient[](1);
            recipients[0] = RevenueRouter.CashRecipient({account: beneficiary, shareBps: 10_000});
            (bool success, bytes memory reason) = address(router)
                .call(abi.encodeCall(RevenueRouter.route, (keccak256("reentry"), settlementAsset, amount, recipients)));
            reentryBlocked = !success && bytes4(reason) == bytes4(keccak256("ReentrancyGuardReentrantCall()"));
            require(reentryBlocked, "reentry was not blocked by guard");
            require(settlementAsset.transferFrom(msg.sender, address(this), amount));
        }
    }

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
                vault = new ResearchVault(
                    IERC20(address(asset)), 6, researcher, address(this), "Research Credit", "RCR"
                );
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

            function testCompoundingDustDoesNotBlockRevenue() public {
                RevenueRouter.CashRecipient[] memory cash = new RevenueRouter.CashRecipient[](0);
                RevenueRouter.CompoundRecipient[] memory compound = new RevenueRouter.CompoundRecipient[](2);
                compound[0] = RevenueRouter.CompoundRecipient({
                    vault: IResearchVault(address(vault)), beneficiary: cashRecipient, shareBps: 1
                });
                compound[1] = RevenueRouter.CompoundRecipient({
                    vault: IResearchVault(address(vault)), beneficiary: researcher, shareBps: 9_999
                });
                vm.startPrank(payer);
                asset.approve(address(router), 1);
                router.routeWithCompounding(keccak256("dust"), IERC20(address(asset)), 1, cash, compound);
                vm.stopPrank();
                assertEq(credit.balanceOf(researcher), 1);
                assertEq(asset.balanceOf(address(router)), 0);
            }

            function testNoopVaultCannotLeaveRevenueTrappedInRouter() public {
                NoopVault noop = new NoopVault(IERC20(address(asset)));
                RevenueRouter.CashRecipient[] memory cash = new RevenueRouter.CashRecipient[](0);
                RevenueRouter.CompoundRecipient[] memory compound = new RevenueRouter.CompoundRecipient[](1);
                compound[0] = RevenueRouter.CompoundRecipient({vault: noop, beneficiary: researcher, shareBps: 10_000});
                uint256 beforePayer = asset.balanceOf(payer);
                bytes32 eventId = keccak256("noop");
                vm.startPrank(payer);
                asset.approve(address(router), 100);
                vm.expectRevert(RevenueRouter.UnsupportedAssetBehavior.selector);
                router.routeWithCompounding(eventId, IERC20(address(asset)), 100, cash, compound);
                vm.stopPrank();
                assertEq(asset.balanceOf(payer), beforePayer);
                assertFalse(router.processedRevenueEvents(payer, eventId));
            }

            function testVaultCallbackCannotReenterRevenueRouting() public {
                ReenteringVault malicious = new ReenteringVault(IERC20(address(asset)), router);
                RevenueRouter.CashRecipient[] memory cash = new RevenueRouter.CashRecipient[](0);
                RevenueRouter.CompoundRecipient[] memory compound = new RevenueRouter.CompoundRecipient[](1);
                compound[0] = RevenueRouter.CompoundRecipient({
                    vault: malicious, beneficiary: researcher, shareBps: 10_000
                });
                vm.startPrank(payer);
                asset.approve(address(router), 100);
                router.routeWithCompounding(keccak256("callback"), IERC20(address(asset)), 100, cash, compound);
                vm.stopPrank();
                assertTrue(malicious.reentryBlocked());
                assertEq(asset.balanceOf(address(malicious)), 100);
                assertEq(asset.balanceOf(address(router)), 0);
            }
        }
