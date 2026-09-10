use xlemma_core::PolicyId;
use xlemma_economics::{
    prepare_wallet_intent, validate_wallet_context, DiscoveryTrust, WalletAction, WalletIntent,
    WalletPolicy, WalletPreflight,
};

const NOW: u64 = 1_788_998_460;
const SAFE: u64 = 9_007_199_254_740_991;

fn fixture() -> (WalletPolicy, DiscoveryTrust, WalletIntent) {
    (
        serde_json::from_str(include_str!("../../../config/wallet-base-sepolia.json")).unwrap(),
        serde_json::from_str(include_str!("../../../examples/wallet/trust.json")).unwrap(),
        serde_json::from_str(include_str!("../../../examples/wallet/fund-intent.json")).unwrap(),
    )
}

#[test]
fn funded_intent_matches_vector_and_keeps_gas_separate() {
    let (policy, trust, intent) = fixture();
    let result = prepare_wallet_intent(&policy, &trust, &intent, NOW).unwrap();
    let expected: WalletPreflight =
        serde_json::from_str(include_str!("../../../examples/wallet/fund-expected.json")).unwrap();
    assert_eq!(result, expected);
    assert_eq!(result.maximum_escrow_allowance_units, 10_000_000);
    assert_eq!(result.maximum_paymaster_allowance_units, 250_000);
    assert_eq!(result.required_wallet_balance_units, 10_250_000);
    assert!(!result.funds_reserved);
    assert!(result.owner_signature_required);
}

#[test]
fn refund_does_not_spend_expected_refund_proceeds() {
    let (policy, trust, _) = fixture();
    let intent =
        serde_json::from_str(include_str!("../../../examples/wallet/refund-intent.json")).unwrap();
    let result = prepare_wallet_intent(&policy, &trust, &intent, NOW).unwrap();
    let expected: WalletPreflight = serde_json::from_str(include_str!(
        "../../../examples/wallet/refund-expected.json"
    ))
    .unwrap();
    assert_eq!(result, expected);
    assert_eq!(result.maximum_escrow_allowance_units, 0);
    assert_eq!(
        result.required_wallet_balance_units,
        result.maximum_paymaster_allowance_units
    );
}

#[test]
fn configuration_cannot_switch_network_contracts_or_signing_authority() {
    let (policy, trust, _) = fixture();
    for (field, replacement) in [
        ("chain_id", serde_json::json!(8453)),
        (
            "usdc_address",
            serde_json::json!("0x0000000000000000000000000000000000000001"),
        ),
        (
            "entry_point_address",
            serde_json::json!(policy.paymaster_address),
        ),
        ("entry_point_version", serde_json::json!("0.8")),
        ("paymaster_address", serde_json::json!(policy.usdc_address)),
        ("signing_mode", serde_json::json!("session_key")),
        ("fee_mode", serde_json::json!("sponsored")),
    ] {
        let mut value = serde_json::to_value(&policy).unwrap();
        value[field] = replacement;
        let changed: WalletPolicy = serde_json::from_value(value).unwrap();
        assert!(
            validate_wallet_context(&changed, &trust).is_err(),
            "{field}"
        );
    }
}

#[test]
fn altered_trust_principal_or_policy_requires_new_intent() {
    let (policy, trust, intent) = fixture();
    let mut changed = trust.clone();
    changed.network = "other-authority".into();
    assert!(prepare_wallet_intent(&policy, &changed, &intent, NOW).is_err());
    changed = trust.clone();
    changed.chain_id = 8453;
    assert!(validate_wallet_context(&policy, &changed).is_err());
    changed = trust.clone();
    changed.usdc_asset = "eip155:84532/erc20:other".into();
    assert!(validate_wallet_context(&policy, &changed).is_err());
    let mut changed = intent.clone();
    changed.principal = "ed25519:unknown".into();
    assert!(prepare_wallet_intent(&policy, &trust, &changed, NOW).is_err());
    changed = intent.clone();
    changed.wallet_address = "0x0000000000000000000000000000000000000008".into();
    assert!(prepare_wallet_intent(&policy, &trust, &changed, NOW).is_err());
    changed = intent.clone();
    changed.policy_id = PolicyId::derive(&"unapproved").unwrap();
    assert!(prepare_wallet_intent(&policy, &trust, &changed, NOW).is_err());
}

#[test]
fn expired_future_excessive_and_overpriced_intents_fail() {
    let (policy, trust, intent) = fixture();
    for (after, until, fee, now) in [
        (NOW, NOW, 1, NOW),
        (NOW + 1, NOW + 2, 1, NOW),
        (NOW, NOW + 301, 1, NOW),
        (NOW, NOW + 1, 0, NOW),
        (NOW, NOW + 1, policy.maximum_fee_units + 1, NOW),
        (NOW, u64::MAX, 1, NOW),
        (NOW, NOW + 1, 1, u64::MAX),
    ] {
        let mut changed = intent.clone();
        changed.valid_after = after;
        changed.valid_until = until;
        changed.maximum_fee_units = fee;
        assert!(prepare_wallet_intent(&policy, &trust, &changed, now).is_err());
    }
}

#[test]
fn generated_amount_boundaries_never_overflow_or_borrow_from_rewards() {
    let (mut policy, trust, mut intent) = fixture();
    policy.maximum_funding_units = SAFE;
    policy.maximum_fee_units = SAFE;
    intent.policy_id = policy.id().unwrap();
    for amount in [
        0,
        1,
        2,
        999_999,
        1_000_000,
        SAFE / 2,
        SAFE - 1,
        SAFE,
        SAFE + 1,
        u64::MAX,
    ] {
        for fee in [
            0,
            1,
            2,
            250_000,
            SAFE / 2,
            SAFE - 1,
            SAFE,
            SAFE + 1,
            u64::MAX,
        ] {
            if let WalletAction::FundRound { amount_units, .. } = &mut intent.action {
                *amount_units = amount;
            }
            intent.maximum_fee_units = fee;
            let result = prepare_wallet_intent(&policy, &trust, &intent, NOW);
            let sum = u128::from(amount) + u128::from(fee);
            if amount > 0 && fee > 0 && sum <= u128::from(SAFE) {
                let result = result.unwrap();
                assert_eq!(u128::from(result.required_wallet_balance_units), sum);
                assert_eq!(result.maximum_escrow_allowance_units, amount);
                assert_eq!(result.maximum_paymaster_allowance_units, fee);
            } else {
                assert!(result.is_err(), "amount={amount} fee={fee}");
            }
        }
    }
}

#[test]
fn nonce_and_fee_changes_change_commitments_without_consuming_authority() {
    let (policy, trust, mut intent) = fixture();
    let original = prepare_wallet_intent(&policy, &trust, &intent, NOW).unwrap();
    // Offline preparation is repeatable; replay consumption belongs to the future signer/outbox.
    assert_eq!(
        original,
        prepare_wallet_intent(&policy, &trust, &intent, NOW).unwrap()
    );
    intent.nonce += 1;
    let changed = prepare_wallet_intent(&policy, &trust, &intent, NOW).unwrap();
    assert_ne!(original.intent_id, changed.intent_id);
    assert_ne!(original.preflight_id, changed.preflight_id);
    intent.maximum_fee_units += 1;
    assert_ne!(
        changed.intent_id,
        prepare_wallet_intent(&policy, &trust, &intent, NOW)
            .unwrap()
            .intent_id
    );
}

#[test]
fn arbitrary_calls_unknown_fields_and_zero_commitments_are_rejected() {
    let (policy, trust, intent) = fixture();
    let mut value = serde_json::to_value(&intent).unwrap();
    value["action"]["kind"] = serde_json::json!("delegatecall");
    assert!(serde_json::from_value::<WalletIntent>(value).is_err());
    let mut value = serde_json::to_value(&intent).unwrap();
    value["action"]["recipient"] = serde_json::json!("attacker");
    assert!(serde_json::from_value::<WalletIntent>(value).is_err());
    let mut value = serde_json::to_value(&intent).unwrap();
    value["action"]["round_id"] = serde_json::json!(format!("xlround:blake3:{}", "0".repeat(64)));
    let changed = serde_json::from_value(value).unwrap();
    assert!(prepare_wallet_intent(&policy, &trust, &changed, NOW).is_err());
}
