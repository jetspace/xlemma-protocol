// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {ERC1155} from "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";

/// @notice Optional token handles for immutable lemma capsules and license editions.
/// @dev Capsule possession never changes mathematical validity or original attribution.
contract LemmaCapsule1155 is ERC1155, AccessControl {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");

    enum TokenKind {
        Unset,
        ProofCapsule,
        LicenseEdition,
        SupportCertificate,
        AccessRight
    }

    mapping(uint256 => TokenKind) public tokenKind;
    mapping(uint256 => bytes32) public manifestRoot;
    mapping(uint256 => address) public originator;

    error InvalidInput();
    error TokenAlreadyDefined();
    error CapsuleIsNonTransferable();

    constructor(string memory baseUri, address administrator) ERC1155(baseUri) {
        if (administrator == address(0)) revert InvalidInput();
        _grantRole(DEFAULT_ADMIN_ROLE, administrator);
        _grantRole(MINTER_ROLE, administrator);
    }

    function mintType(
        address to,
        uint256 tokenId,
        uint256 amount,
        TokenKind kind,
        bytes32 manifestRoot_,
        address originator_,
        bytes calldata data
    ) external onlyRole(MINTER_ROLE) {
        if (
            to == address(0) || originator_ == address(0) || amount == 0 ||
            kind == TokenKind.Unset || manifestRoot_ == bytes32(0)
        ) revert InvalidInput();
        if (kind == TokenKind.ProofCapsule && amount != 1) revert InvalidInput();
        if (tokenKind[tokenId] != TokenKind.Unset) revert TokenAlreadyDefined();
        tokenKind[tokenId] = kind;
        manifestRoot[tokenId] = manifestRoot_;
        originator[tokenId] = originator_;
        _mint(to, tokenId, amount, data);
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        override(ERC1155, AccessControl)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }

    function _update(
        address from,
        address to,
        uint256[] memory ids,
        uint256[] memory values
    ) internal override {
        if (from != address(0) && to != address(0)) {
            for (uint256 i = 0; i < ids.length; ++i) {
                if (
                    tokenKind[ids[i]] == TokenKind.ProofCapsule ||
                    tokenKind[ids[i]] == TokenKind.SupportCertificate
                ) {
                    revert CapsuleIsNonTransferable();
                }
            }
        }
        super._update(from, to, ids, values);
    }
}
