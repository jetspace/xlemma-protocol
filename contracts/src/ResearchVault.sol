// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {ResearcherCredit} from "./ResearcherCredit.sol";

/// @notice Holds neutral settlement backing for one researcher's closed-loop credits.
contract ResearchVault is AccessControl, ReentrancyGuard {
    using SafeERC20 for IERC20;

    bytes32 public constant SETTLER_ROLE = keccak256("SETTLER_ROLE");
    bytes32 public constant REVENUE_ROUTER_ROLE = keccak256("REVENUE_ROUTER_ROLE");

    struct Authorization {
        address payer;
        address payee;
        uint256 maximum;
        uint64 expiresAt;
        bool closed;
    }

    IERC20 public immutable settlementAsset;
    ResearcherCredit public immutable credit;
    address public immutable researcher;
    uint8 public immutable assetDecimals;

    mapping(bytes32 => Authorization) public authorizations;

    event Deposited(address indexed payer, address indexed beneficiary, uint256 amount);
    event Authorized(
        bytes32 indexed authorizationId,
        address indexed payer,
        address indexed payee,
        uint256 maximum,
        uint64 expiresAt
    );
    event Settled(
        bytes32 indexed authorizationId,
        address indexed payee,
        uint256 actual,
        uint256 refunded
    );
    event RevenueCompounded(address indexed beneficiary, uint256 amount);
    event Redeemed(address indexed owner, address indexed recipient, uint256 amount);

    error InvalidAddress();
    error InvalidAmount();
    error InvalidDecimals();
    error AuthorizationExists();
    error AuthorizationClosed();
    error AuthorizationExpired();
    error ExceedsMaximum();
    error Insolvent();

    constructor(
        IERC20 settlementAsset_,
        uint8 assetDecimals_,
        address researcher_,
        address administrator,
        string memory creditName,
        string memory creditSymbol
    ) {
        if (
            address(settlementAsset_) == address(0) || researcher_ == address(0) ||
            administrator == address(0)
        ) revert InvalidAddress();
        if (IERC20Metadata(address(settlementAsset_)).decimals() != assetDecimals_) {
            revert InvalidDecimals();
        }

        settlementAsset = settlementAsset_;
        assetDecimals = assetDecimals_;
        researcher = researcher_;
        credit = new ResearcherCredit(
            creditName,
            creditSymbol,
            assetDecimals_,
            address(this),
            administrator
        );
        _grantRole(DEFAULT_ADMIN_ROLE, administrator);
        _grantRole(SETTLER_ROLE, administrator);
        _grantRole(REVENUE_ROUTER_ROLE, administrator);
    }

    function deposit(uint256 amount, address beneficiary) external nonReentrant {
        if (amount == 0) revert InvalidAmount();
        if (beneficiary == address(0)) revert InvalidAddress();
        settlementAsset.safeTransferFrom(msg.sender, address(this), amount);
        credit.mint(beneficiary, amount);
        _assertSolvent();
        emit Deposited(msg.sender, beneficiary, amount);
    }

    /// @notice Locks credits for one idempotent x402 authorization and binds
    ///         settlement to the quoted payee/router.
    function authorize(
        bytes32 authorizationId,
        address payee,
        uint256 maximum,
        uint64 expiresAt
    ) external nonReentrant {
        if (maximum == 0 || expiresAt <= block.timestamp) revert InvalidAmount();
        if (authorizationId == bytes32(0) || payee == address(0)) revert InvalidAddress();
        if (authorizations[authorizationId].payer != address(0)) revert AuthorizationExists();

        credit.transferFrom(msg.sender, address(this), maximum);
        authorizations[authorizationId] = Authorization({
            payer: msg.sender,
            payee: payee,
            maximum: maximum,
            expiresAt: expiresAt,
            closed: false
        });
        emit Authorized(authorizationId, msg.sender, payee, maximum, expiresAt);
    }

    /// @notice Burns only actual usage, pays the pre-authorized payee, and
    ///         returns the unused maximum to the researcher.
    function settle(bytes32 authorizationId, uint256 actual)
        external
        onlyRole(SETTLER_ROLE)
        nonReentrant
    {
        Authorization storage authorization = authorizations[authorizationId];
        if (authorization.payer == address(0)) revert InvalidAddress();
        if (authorization.closed) revert AuthorizationClosed();
        if (block.timestamp > authorization.expiresAt) revert AuthorizationExpired();
        if (actual > authorization.maximum) revert ExceedsMaximum();

        authorization.closed = true;
        uint256 refund = authorization.maximum - actual;
        if (actual != 0) {
            credit.burn(address(this), actual);
            settlementAsset.safeTransfer(authorization.payee, actual);
        }
        if (refund != 0) {
            credit.transfer(authorization.payer, refund);
        }
        _assertSolvent();
        emit Settled(authorizationId, authorization.payee, actual, refund);
    }

    function cancelExpired(bytes32 authorizationId) external nonReentrant {
        Authorization storage authorization = authorizations[authorizationId];
        if (authorization.payer == address(0)) revert InvalidAddress();
        if (authorization.closed) revert AuthorizationClosed();
        if (block.timestamp <= authorization.expiresAt) revert AuthorizationExpired();
        authorization.closed = true;
        credit.transfer(authorization.payer, authorization.maximum);
        emit Settled(authorizationId, authorization.payee, 0, authorization.maximum);
    }

    /// @notice Converts already-settled external revenue into newly backed credits.
    function compoundRevenue(uint256 amount, address beneficiary)
        external
        onlyRole(REVENUE_ROUTER_ROLE)
        nonReentrant
    {
        if (amount == 0) revert InvalidAmount();
        if (beneficiary == address(0)) revert InvalidAddress();
        settlementAsset.safeTransferFrom(msg.sender, address(this), amount);
        credit.mint(beneficiary, amount);
        _assertSolvent();
        emit RevenueCompounded(beneficiary, amount);
    }

    /// @dev Optional redemption path. Production deployments may restrict this
    /// further based on the legal treatment of the credit program.
    function redeem(uint256 amount, address recipient) external nonReentrant {
        if (amount == 0) revert InvalidAmount();
        if (recipient == address(0)) revert InvalidAddress();
        credit.burn(msg.sender, amount);
        settlementAsset.safeTransfer(recipient, amount);
        _assertSolvent();
        emit Redeemed(msg.sender, recipient, amount);
    }

    function backing() public view returns (uint256) {
        return settlementAsset.balanceOf(address(this));
    }

    function isSolvent() public view returns (bool) {
        return backing() >= credit.totalSupply();
    }

    function _assertSolvent() internal view {
        if (!isSolvent()) revert Insolvent();
    }
}
