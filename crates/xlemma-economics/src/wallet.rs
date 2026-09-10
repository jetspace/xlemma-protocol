//! Read-only wallet intent validation. This module never signs, reserves funds,
//! creates a UserOperation, observes settlement or changes discovery state.
use crate::{DiscoveryTrust, RewardCategory};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use xlemma_core::{ArtifactId, DiscoveryRoundId, PolicyId, ReceiptId, ResearcherId, XLMP_VERSION};

pub const BASE_SEPOLIA_CHAIN_ID: u64 = 84532;
pub const BASE_SEPOLIA_USDC: &str = "0x036cbd53842c5426634e7929541ec2318f3dcf7e";
pub const ENTRY_POINT_V07: &str = "0x0000000071727de22e5e9d8baf0edac6f37da032";
pub const CIRCLE_TESTNET_PAYMASTER_V07: &str = "0x31be08d380a21fc740883c0bc434fcfc88740b58";
const SAFE: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletPolicy {
    pub protocol_version: String,
    pub profile: String,
    pub chain_id: u64,
    pub usdc_address: String,
    pub entry_point_version: String,
    pub entry_point_address: String,
    pub paymaster_address: String,
    pub signing_mode: String,
    pub fee_mode: String,
    pub maximum_funding_units: u64,
    /// Maximum total USDC fee including provider charges, not a gas estimate.
    pub maximum_fee_units: u64,
    pub maximum_intent_lifetime_seconds: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WalletAction {
    FundRound {
        round_id: DiscoveryRoundId,
        category: RewardCategory,
        amount_units: u64,
        mandate_root: ArtifactId,
    },
    Refund {
        round_id: DiscoveryRoundId,
        category: RewardCategory,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletIntent {
    pub protocol_version: String,
    pub policy_id: PolicyId,
    pub trust_root: PolicyId,
    pub principal: String,
    pub wallet_address: String,
    /// Application intent nonce; not an ERC-4337 account nonce.
    pub nonce: u64,
    pub valid_after: u64,
    pub valid_until: u64,
    pub maximum_fee_units: u64,
    pub action: WalletAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletPreflight {
    pub protocol_version: String,
    pub status: String,
    /// Content commitment to this draft; not evidence of an external action.
    pub preflight_id: ReceiptId,
    pub policy_id: PolicyId,
    pub trust_root: PolicyId,
    pub intent_id: ReceiptId,
    pub researcher_id: ResearcherId,
    pub chain_id: u64,
    pub wallet_address: String,
    pub escrow_address: String,
    pub usdc_asset: String,
    pub entry_point_address: String,
    pub paymaster_address: String,
    pub method: String,
    pub arguments: Vec<String>,
    pub maximum_escrow_allowance_units: u64,
    pub maximum_paymaster_allowance_units: u64,
    pub required_wallet_balance_units: u64,
    pub checked_at: u64,
    pub expires_at: u64,
    pub owner_signature_required: bool,
    pub funds_reserved: bool,
}

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("invalid wallet input: {0}")]
    Invalid(&'static str),
    #[error("wallet identity encoding failed")]
    Identity,
}

fn require(value: bool, reason: &'static str) -> Result<(), WalletError> {
    if value {
        Ok(())
    } else {
        Err(WalletError::Invalid(reason))
    }
}

fn address(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && value[2..].bytes().any(|b| b != b'0')
}

impl WalletPolicy {
    pub fn validate(&self) -> Result<(), WalletError> {
        require(
            self.protocol_version == XLMP_VERSION
                && self.profile == "base-sepolia-circle-v07-reference"
                && self.chain_id == BASE_SEPOLIA_CHAIN_ID
                && self.usdc_address == BASE_SEPOLIA_USDC
                && self.entry_point_version == "0.7"
                && self.entry_point_address == ENTRY_POINT_V07
                && self.paymaster_address == CIRCLE_TESTNET_PAYMASTER_V07,
            "unsupported network or contract set",
        )?;
        require(
            self.signing_mode == "owner_only" && self.fee_mode == "self_funded",
            "delegation and sponsorship require a separate qualified adapter",
        )?;
        require(
            self.maximum_funding_units > 0
                && self.maximum_funding_units <= SAFE
                && self.maximum_fee_units > 0
                && self.maximum_fee_units <= SAFE
                && (1..=3600).contains(&self.maximum_intent_lifetime_seconds),
            "invalid amount or lifetime cap",
        )
    }

    pub fn id(&self) -> Result<PolicyId, WalletError> {
        self.validate()?;
        PolicyId::derive(self).map_err(|_| WalletError::Identity)
    }
}

fn category_index(category: &RewardCategory) -> u8 {
    match category {
        RewardCategory::FoundationalResearch => 0,
        RewardCategory::Discovery => 1,
        RewardCategory::Formalization => 2,
        RewardCategory::ProofImprovement => 3,
        RewardCategory::Replication => 4,
        RewardCategory::ResearchTools => 5,
        RewardCategory::NegativeResult => 6,
    }
}

pub fn validate_wallet_context(
    policy: &WalletPolicy,
    trust: &DiscoveryTrust,
) -> Result<(PolicyId, PolicyId), WalletError> {
    let policy_id = policy.id()?;
    trust
        .validate()
        .map_err(|_| WalletError::Invalid("invalid discovery trust"))?;
    let trust_root = trust.root().map_err(|_| WalletError::Identity)?;
    let usdc_asset = format!("eip155:{}/erc20:{}", policy.chain_id, policy.usdc_address);
    require(
        trust.chain_id == policy.chain_id
            && trust.usdc_asset == usdc_asset
            && address(&trust.escrow_address)
            && ![
                policy.usdc_address.as_str(),
                policy.entry_point_address.as_str(),
                policy.paymaster_address.as_str(),
            ]
            .contains(&trust.escrow_address.as_str()),
        "discovery settlement network or asset mismatch",
    )?;
    Ok((policy_id, trust_root))
}

fn abi_digest(id: &str) -> Result<String, WalletError> {
    let digest = id.rsplit(':').next().ok_or(WalletError::Identity)?;
    require(
        digest.bytes().any(|b| b != b'0'),
        "zero contract commitment",
    )?;
    Ok(format!("0x{digest}"))
}

pub fn prepare_wallet_intent(
    policy: &WalletPolicy,
    trust: &DiscoveryTrust,
    intent: &WalletIntent,
    now: u64,
) -> Result<WalletPreflight, WalletError> {
    let (policy_id, trust_root) = validate_wallet_context(policy, trust)?;
    let usdc_asset = trust.usdc_asset.clone();
    require(
        intent.protocol_version == XLMP_VERSION
            && intent.policy_id == policy_id
            && intent.trust_root == trust_root,
        "intent policy or trust root mismatch",
    )?;
    let principal = trust
        .principals
        .get(&intent.principal)
        .ok_or(WalletError::Invalid("unregistered principal"))?;
    require(
        address(&intent.wallet_address)
            && intent
                .wallet_address
                .eq_ignore_ascii_case(&principal.payout_address)
            && ![
                policy.usdc_address.as_str(),
                policy.entry_point_address.as_str(),
                policy.paymaster_address.as_str(),
            ]
            .contains(&intent.wallet_address.as_str()),
        "wallet does not match registered payout address",
    )?;
    require(
        intent.nonce <= SAFE
            && now <= SAFE
            && intent.valid_until <= SAFE
            && intent.valid_after <= now
            && now < intent.valid_until
            && intent.valid_until - intent.valid_after <= policy.maximum_intent_lifetime_seconds,
        "expired, future or excessive intent lifetime",
    )?;
    require(
        intent.maximum_fee_units > 0 && intent.maximum_fee_units <= policy.maximum_fee_units,
        "fee exceeds policy cap",
    )?;
    let (method, arguments, amount) = match &intent.action {
        WalletAction::FundRound {
            round_id,
            category,
            amount_units,
            mandate_root,
        } => {
            round_id.validate().map_err(|_| WalletError::Identity)?;
            mandate_root.validate().map_err(|_| WalletError::Identity)?;
            require(
                *amount_units > 0 && *amount_units <= policy.maximum_funding_units,
                "funding exceeds policy cap",
            )?;
            (
                "fund(bytes32,uint8,uint256,bytes32)",
                vec![
                    abi_digest(round_id.as_str())?,
                    category_index(category).to_string(),
                    amount_units.to_string(),
                    abi_digest(mandate_root.as_str())?,
                ],
                *amount_units,
            )
        }
        WalletAction::Refund { round_id, category } => {
            round_id.validate().map_err(|_| WalletError::Identity)?;
            (
                "refund(bytes32,uint8)",
                vec![
                    abi_digest(round_id.as_str())?,
                    category_index(category).to_string(),
                ],
                0,
            )
        }
    };
    let required = amount
        .checked_add(intent.maximum_fee_units)
        .filter(|value| *value <= SAFE)
        .ok_or(WalletError::Invalid("combined debit exceeds safe amount"))?;
    let intent_id =
        ReceiptId::derive(&("wallet-intent-v1", intent)).map_err(|_| WalletError::Identity)?;
    let mut result = WalletPreflight {
        protocol_version: XLMP_VERSION.into(),
        status: "prepared_unsigned".into(),
        preflight_id: intent_id.clone(),
        policy_id,
        trust_root,
        intent_id,
        researcher_id: principal.researcher_id.clone(),
        chain_id: policy.chain_id,
        wallet_address: intent.wallet_address.clone(),
        escrow_address: trust.escrow_address.clone(),
        usdc_asset,
        entry_point_address: policy.entry_point_address.clone(),
        paymaster_address: policy.paymaster_address.clone(),
        method: method.into(),
        arguments,
        maximum_escrow_allowance_units: amount,
        maximum_paymaster_allowance_units: intent.maximum_fee_units,
        required_wallet_balance_units: required,
        checked_at: now,
        expires_at: intent.valid_until,
        owner_signature_required: true,
        funds_reserved: false,
    };
    // Serialize the complete unsigned draft without its own identity field.
    let mut value = serde_json::to_value(&result).map_err(|_| WalletError::Identity)?;
    value
        .as_object_mut()
        .ok_or(WalletError::Identity)?
        .remove("preflight_id");
    result.preflight_id =
        ReceiptId::derive(&("wallet-preflight-v1", value)).map_err(|_| WalletError::Identity)?;
    Ok(result)
}
