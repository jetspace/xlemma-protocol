// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {NodeBondRegistry} from "../src/NodeBondRegistry.sol";
import {MockUSDC} from "./mocks/MockUSDC.sol";

contract NodeBondRegistryTest is Test {
    MockUSDC internal asset;
    NodeBondRegistry internal registry;
    address internal nodeOwner = makeAddr("node-owner");
    address internal slashRecipient = makeAddr("slash-recipient");
    bytes32 internal constant NODE_ID = keccak256("node");
    bytes32 internal constant OPERATOR_ID = keccak256("operator");
    bytes32 internal constant ROLES_ROOT = keccak256("roles");

    function setUp() public {
        asset = new MockUSDC();
        registry = new NodeBondRegistry(IERC20(address(asset)), address(this), 1 days);
        asset.mint(nodeOwner, 1_000e6);
    }

    function _registerAndBond(uint256 amount) internal {
        vm.startPrank(nodeOwner);
        registry.register(NODE_ID, OPERATOR_ID, ROLES_ROOT);
        asset.approve(address(registry), amount);
        registry.bond(NODE_ID, amount);
        vm.stopPrank();
    }

    function testOperatorClusterIsImmutableForNodeId() public {
        vm.startPrank(nodeOwner);
        registry.register(NODE_ID, OPERATOR_ID, ROLES_ROOT);
        vm.expectRevert(NodeBondRegistry.InvalidState.selector);
        registry.register(NODE_ID, keccak256("different-operator"), ROLES_ROOT);
        vm.stopPrank();
    }

    function testUnbondUsesDelayAndDeactivatesNode() public {
        _registerAndBond(500e6);

        vm.prank(nodeOwner);
        registry.requestUnbond(NODE_ID, 200e6);
        vm.prank(nodeOwner);
        vm.expectRevert(NodeBondRegistry.InvalidState.selector);
        registry.register(NODE_ID, OPERATOR_ID, ROLES_ROOT);
        vm.prank(nodeOwner);
        vm.expectRevert(NodeBondRegistry.DelayOpen.selector);
        registry.withdrawUnbond(NODE_ID);

        vm.warp(block.timestamp + 1 days);
        vm.prank(nodeOwner);
        registry.withdrawUnbond(NODE_ID);
        assertEq(asset.balanceOf(nodeOwner), 700e6);
        assertFalse(registry.getNode(NODE_ID).active);
    }

    function testObjectiveSlashReducesBondAndDisablesNode() public {
        _registerAndBond(500e6);

        registry.slash(NODE_ID, 100e6, keccak256("equivocation-evidence"), slashRecipient);

        NodeBondRegistry.Node memory node = registry.getNode(NODE_ID);
        assertEq(node.bonded, 400e6);
        assertFalse(node.active);
        assertEq(asset.balanceOf(slashRecipient), 100e6);
    }
}
