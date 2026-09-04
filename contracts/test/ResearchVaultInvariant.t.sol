// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {StdInvariant} from "forge-std/StdInvariant.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ResearchVault} from "../src/ResearchVault.sol";
import {ResearcherCredit} from "../src/ResearcherCredit.sol";
import {MockUSDC} from "./mocks/MockUSDC.sol";

contract ResearchVaultHandler is Test {
    MockUSDC public immutable asset;
    ResearchVault public immutable vault;
    ResearcherCredit public immutable credit;

    bytes32 public activeAuthorization;
    uint256 public activeMaximum;
    uint256 internal nonce;
    address internal constant PAYEE = address(0xBEEF);

    constructor(MockUSDC asset_, ResearchVault vault_) {
        asset = asset_;
        vault = vault_;
        credit = vault_.credit();
        asset.approve(address(vault), type(uint256).max);
        credit.approve(address(vault), type(uint256).max);
    }

    function deposit(uint96 rawAmount) external {
        uint256 amount = bound(uint256(rawAmount), 1, 1_000_000e6);
        asset.mint(address(this), amount);
        vault.deposit(amount, address(this));
    }

    function authorize(uint96 rawMaximum) external {
        if (activeAuthorization != bytes32(0)) return;
        uint256 balance = credit.balanceOf(address(this));
        if (balance == 0) return;
        uint256 maximum = bound(uint256(rawMaximum), 1, balance);
        bytes32 authorizationId = keccak256(
            abi.encode("invariant-authorization", ++nonce)
        );
        vault.authorize(
            authorizationId,
            PAYEE,
            maximum,
            uint64(block.timestamp + 365 days)
        );
        activeAuthorization = authorizationId;
        activeMaximum = maximum;
    }

    function settle(uint96 rawActual) external {
        if (activeAuthorization == bytes32(0)) return;
        uint256 actual = bound(uint256(rawActual), 0, activeMaximum);
        vault.settle(activeAuthorization, actual);
        activeAuthorization = bytes32(0);
        activeMaximum = 0;
    }

    function compound(uint96 rawAmount) external {
        uint256 amount = bound(uint256(rawAmount), 1, 1_000_000e6);
        asset.mint(address(this), amount);
        vault.compoundRevenue(amount, address(this));
    }

    function redeem(uint96 rawAmount) external {
        uint256 balance = credit.balanceOf(address(this));
        if (balance == 0) return;
        uint256 amount = bound(uint256(rawAmount), 1, balance);
        vault.redeem(amount, address(this));
    }
}

contract ResearchVaultInvariantTest is StdInvariant, Test {
    MockUSDC internal asset;
    ResearchVault internal vault;
    ResearchVaultHandler internal handler;

    function setUp() public {
        asset = new MockUSDC();
        vault = new ResearchVault(
            IERC20(address(asset)),
            6,
            address(this),
            address(this),
            "Invariant Research Credit",
            "iRCR"
        );
        handler = new ResearchVaultHandler(asset, vault);
        vault.grantRole(vault.SETTLER_ROLE(), address(handler));
        vault.grantRole(vault.REVENUE_ROUTER_ROLE(), address(handler));
        targetContract(address(handler));
    }

    function invariantBackingAlwaysCoversOutstandingCredits() public view {
        assertGe(asset.balanceOf(address(vault)), vault.credit().totalSupply());
        assertTrue(vault.isSolvent());
    }
}
