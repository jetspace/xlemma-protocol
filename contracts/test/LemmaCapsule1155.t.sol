// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {LemmaCapsule1155} from "../src/LemmaCapsule1155.sol";

contract LemmaCapsule1155Test is Test {
    LemmaCapsule1155 internal token;
    address internal originator = makeAddr("originator");
    address internal holder = makeAddr("holder");
    address internal buyer = makeAddr("buyer");

    function setUp() public {
        token = new LemmaCapsule1155("ipfs://{id}", address(this));
    }

    function testProofCapsuleIsUniqueAndNonTransferable() public {
        uint256 tokenId = 1;
        token.mintType(
            holder, tokenId, 1, LemmaCapsule1155.TokenKind.ProofCapsule, keccak256("manifest"), originator, ""
        );

        vm.prank(holder);
        vm.expectRevert(LemmaCapsule1155.CapsuleIsNonTransferable.selector);
        token.safeTransferFrom(holder, buyer, tokenId, 1, "");
    }

    function testLicenseEditionCanTransferWithoutChangingOriginator() public {
        uint256 tokenId = 2;
        token.mintType(
            holder,
            tokenId,
            100,
            LemmaCapsule1155.TokenKind.LicenseEdition,
            keccak256("license-manifest"),
            originator,
            ""
        );

        vm.prank(holder);
        token.safeTransferFrom(holder, buyer, tokenId, 10, "");

        assertEq(token.balanceOf(buyer, tokenId), 10);
        assertEq(token.originator(tokenId), originator);
    }
}
