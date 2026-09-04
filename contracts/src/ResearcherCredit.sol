// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";

/// @notice Fully backed, restricted-transfer research service credit.
/// @dev It is not a proof-validity vote and does not carry profit rights.
contract ResearcherCredit is ERC20, AccessControl {
    bytes32 public constant VAULT_ROLE = keccak256("VAULT_ROLE");
    bytes32 public constant TRANSFER_ENDPOINT_ROLE = keccak256("TRANSFER_ENDPOINT_ROLE");

    error InvalidAddress();
    error RestrictedTransfer(address from, address to);

    uint8 private immutable _creditDecimals;

    constructor(
        string memory name_,
        string memory symbol_,
        uint8 decimals_,
        address vault,
        address administrator
    ) ERC20(name_, symbol_) {
        if (vault == address(0) || administrator == address(0)) revert InvalidAddress();
        _creditDecimals = decimals_;
        _grantRole(DEFAULT_ADMIN_ROLE, administrator);
        _grantRole(VAULT_ROLE, vault);
        _grantRole(TRANSFER_ENDPOINT_ROLE, vault);
    }

    function decimals() public view override returns (uint8) {
        return _creditDecimals;
    }

    function mint(address to, uint256 amount) external onlyRole(VAULT_ROLE) {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyRole(VAULT_ROLE) {
        _burn(from, amount);
    }

    function setTransferEndpoint(address endpoint, bool allowed)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        if (endpoint == address(0)) revert InvalidAddress();
        if (allowed) {
            _grantRole(TRANSFER_ENDPOINT_ROLE, endpoint);
        } else {
            _revokeRole(TRANSFER_ENDPOINT_ROLE, endpoint);
        }
    }

    function _update(address from, address to, uint256 value) internal override {
        bool mintOrBurn = from == address(0) || to == address(0);
        bool endpointTransfer =
            hasRole(TRANSFER_ENDPOINT_ROLE, from) || hasRole(TRANSFER_ENDPOINT_ROLE, to);
        if (!mintOrBurn && !endpointTransfer) {
            revert RestrictedTransfer(from, to);
        }
        super._update(from, to, value);
    }
}
