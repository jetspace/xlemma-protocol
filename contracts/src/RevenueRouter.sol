// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

interface IResearchVault {
    function settlementAsset() external view returns (IERC20);
    function compoundRevenue(uint256 amount, address beneficiary) external;
}

/// @notice Routes realized external revenue. It does not distribute unrealized token gains.
contract RevenueRouter is ReentrancyGuard {
    using SafeERC20 for IERC20;

    uint256 private constant BPS = 10_000;

    struct CashRecipient {
        address account;
        uint16 shareBps;
    }

    struct CompoundRecipient {
        IResearchVault vault;
        address beneficiary;
        uint16 shareBps;
    }

    mapping(address => mapping(bytes32 => bool)) public processedRevenueEvents;

    event RevenueRouted(
        bytes32 indexed revenueEventId,
        address indexed asset,
        uint256 amount,
        uint256 cashRecipientCount,
        uint256 compoundRecipientCount
    );

    error InvalidEvent();
    error InvalidShares();
    error InvalidRecipient();
    error IncompatibleVaultAsset();
    error UnsupportedAssetBehavior();

    function route(bytes32 revenueEventId, IERC20 asset, uint256 amount, CashRecipient[] calldata recipients)
        external
        nonReentrant
    {
        CompoundRecipient[] memory noCompound = new CompoundRecipient[](0);
        _route(revenueEventId, asset, amount, recipients, noCompound);
    }

    /// @notice Splits external cash revenue and atomically converts selected
    ///         allocations into newly backed researcher credits.
    function routeWithCompounding(
        bytes32 revenueEventId,
        IERC20 asset,
        uint256 amount,
        CashRecipient[] calldata cashRecipients,
        CompoundRecipient[] calldata compoundRecipients
    ) external nonReentrant {
        _route(revenueEventId, asset, amount, cashRecipients, compoundRecipients);
    }

    function _route(
        bytes32 revenueEventId,
        IERC20 asset,
        uint256 amount,
        CashRecipient[] calldata cashRecipients,
        CompoundRecipient[] memory compoundRecipients
    ) internal {
        if (
            revenueEventId == bytes32(0) || processedRevenueEvents[msg.sender][revenueEventId]
                || address(asset) == address(0)
        ) revert InvalidEvent();
        if (amount == 0 || cashRecipients.length + compoundRecipients.length == 0) {
            revert InvalidShares();
        }

        uint256 totalBps = 0;
        for (uint256 i = 0; i < cashRecipients.length; ++i) {
            if (cashRecipients[i].account == address(0)) revert InvalidRecipient();
            totalBps += cashRecipients[i].shareBps;
        }
        for (uint256 i = 0; i < compoundRecipients.length; ++i) {
            if (address(compoundRecipients[i].vault) == address(0) || compoundRecipients[i].beneficiary == address(0)) {
                revert InvalidRecipient();
            }
            if (address(compoundRecipients[i].vault.settlementAsset()) != address(asset)) {
                revert IncompatibleVaultAsset();
            }
            totalBps += compoundRecipients[i].shareBps;
        }
        if (totalBps != BPS) revert InvalidShares();

        processedRevenueEvents[msg.sender][revenueEventId] = true;
        uint256 beforeBalance = asset.balanceOf(address(this));
        asset.safeTransferFrom(msg.sender, address(this), amount);
        if (asset.balanceOf(address(this)) != beforeBalance + amount) {
            revert UnsupportedAssetBehavior();
        }

        uint256 paid = 0;
        uint256 cursor = 0;
        uint256 recipientCount = cashRecipients.length + compoundRecipients.length;
        for (uint256 i = 0; i < cashRecipients.length; ++i) {
            uint256 allocation = ++cursor == recipientCount ? amount - paid : amount * cashRecipients[i].shareBps / BPS;
            paid += allocation;
            _pushExact(asset, cashRecipients[i].account, allocation);
        }
        for (uint256 i = 0; i < compoundRecipients.length; ++i) {
            uint256 allocation =
                ++cursor == recipientCount ? amount - paid : amount * compoundRecipients[i].shareBps / BPS;
            paid += allocation;
            if (allocation == 0) continue;
            uint256 beforeCompound = asset.balanceOf(address(this));
            asset.forceApprove(address(compoundRecipients[i].vault), allocation);
            compoundRecipients[i].vault.compoundRevenue(allocation, compoundRecipients[i].beneficiary);
            asset.forceApprove(address(compoundRecipients[i].vault), 0);
            if (asset.balanceOf(address(this)) + allocation != beforeCompound) {
                revert UnsupportedAssetBehavior();
            }
        }

        if (asset.balanceOf(address(this)) != beforeBalance) revert UnsupportedAssetBehavior();

        emit RevenueRouted(revenueEventId, address(asset), amount, cashRecipients.length, compoundRecipients.length);
    }

    function _pushExact(IERC20 asset, address recipient, uint256 amount) internal {
        uint256 beforeRouter = asset.balanceOf(address(this));
        uint256 beforeRecipient = asset.balanceOf(recipient);
        asset.safeTransfer(recipient, amount);
        if (
            asset.balanceOf(address(this)) + amount != beforeRouter
                || asset.balanceOf(recipient) != beforeRecipient + amount
        ) revert UnsupportedAssetBehavior();
    }
}
