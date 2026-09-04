// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ResearchVault} from "./ResearchVault.sol";

contract ResearchVaultFactory {
    event ResearchVaultCreated(
        bytes32 indexed researcherId,
        address indexed researcher,
        address vault,
        address credit
    );

    mapping(bytes32 => address) public vaultByResearcherId;

    error ResearcherAlreadyRegistered();

    function createVault(
        bytes32 researcherId,
        IERC20 settlementAsset,
        uint8 assetDecimals,
        address researcher,
        address administrator,
        string calldata creditName,
        string calldata creditSymbol
    ) external returns (ResearchVault vault) {
        if (vaultByResearcherId[researcherId] != address(0)) {
            revert ResearcherAlreadyRegistered();
        }
        vault = new ResearchVault(
            settlementAsset,
            assetDecimals,
            researcher,
            administrator,
            creditName,
            creditSymbol
        );
        vaultByResearcherId[researcherId] = address(vault);
        emit ResearchVaultCreated(researcherId, researcher, address(vault), address(vault.credit()));
    }
}
