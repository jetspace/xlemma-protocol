// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ResearchVault} from "./ResearchVault.sol";

contract ResearchVaultFactory {
    event ResearchVaultCreated(bytes32 indexed researcherId, address indexed researcher, address vault, address credit);

    /// ResearcherIDs are namespaced by the self-registering account so an
    /// unrelated address cannot front-run and occupy another account's slot.
    mapping(address => mapping(bytes32 => address)) public vaultByResearcher;

    error ResearcherAlreadyRegistered();
    error UnauthorizedRegistration();

    function createVault(
        bytes32 researcherId,
        IERC20 settlementAsset,
        uint8 assetDecimals,
        address researcher,
        address administrator,
        string calldata creditName,
        string calldata creditSymbol
    ) external returns (ResearchVault vault) {
        if (
            researcherId == bytes32(0) || researcher == address(0) || msg.sender != researcher
                || administrator != researcher
        ) revert UnauthorizedRegistration();
        if (vaultByResearcher[researcher][researcherId] != address(0)) {
            revert ResearcherAlreadyRegistered();
        }
        vault = new ResearchVault(settlementAsset, assetDecimals, researcher, administrator, creditName, creditSymbol);
        vaultByResearcher[researcher][researcherId] = address(vault);
        emit ResearchVaultCreated(researcherId, researcher, address(vault), address(vault.credit()));
    }
}
