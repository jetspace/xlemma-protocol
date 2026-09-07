// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ResearchVaultFactory} from "../src/ResearchVaultFactory.sol";
import {MockUSDC} from "./mocks/MockUSDC.sol";

contract ResearchVaultFactoryTest is Test {
    ResearchVaultFactory internal factory;
    MockUSDC internal asset;

    address internal researcher = makeAddr("researcher");
    address internal attacker = makeAddr("attacker");
    bytes32 internal constant RESEARCHER_ID = keccak256("researcher-id");

    function setUp() public {
        factory = new ResearchVaultFactory();
        asset = new MockUSDC();
    }

    function testAnUnrelatedAccountCannotSquatAnotherResearchersNamespace() public {
        vm.prank(attacker);
        factory.createVault(RESEARCHER_ID, IERC20(address(asset)), 6, attacker, attacker, "Attacker Credit", "ACR");

        vm.prank(researcher);
        factory.createVault(RESEARCHER_ID, IERC20(address(asset)), 6, researcher, researcher, "Research Credit", "RCR");

        assertTrue(factory.vaultByResearcher(attacker, RESEARCHER_ID) != address(0));
        assertTrue(factory.vaultByResearcher(researcher, RESEARCHER_ID) != address(0));
        assertTrue(
            factory.vaultByResearcher(attacker, RESEARCHER_ID) != factory.vaultByResearcher(researcher, RESEARCHER_ID)
        );
    }

    function testCallerCannotRegisterForAnotherResearcher() public {
        vm.prank(attacker);
        vm.expectRevert(ResearchVaultFactory.UnauthorizedRegistration.selector);
        factory.createVault(RESEARCHER_ID, IERC20(address(asset)), 6, researcher, researcher, "Research Credit", "RCR");
    }
}
