//! Spot and forward pricing for proof-production and verification services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xlemma_core::{Amount, PolicyId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchService {
    AstraGeneration,
    LeanBuild,
    OfficialKernelCheck,
    IndependentCheck,
    NoveltyReview,
    HumanExpertReview,
    Storage,
    ChallengeReserve,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceOffer {
    pub offer_id: String,
    pub provider_id: String,
    pub service: ResearchService,
    pub model_or_checker: Option<String>,
    pub hardware_class: Option<String>,
    pub domain: Option<String>,
    pub delivery_start: DateTime<Utc>,
    pub delivery_end: DateTime<Utc>,
    /// Price in settlement minor units for `quantity_scale` service units.
    pub price_minor_units: u128,
    pub quantity_scale: u64,
    pub capacity_units: u64,
    pub p95_latency_ms: u64,
    pub collateral_reference: String,
    pub quote_asset: String,
    pub quote_decimals: u8,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceSuccessEstimate {
    pub offer_id: String,
    /// Independently calibrated completion probability in basis points.
    pub completion_probability_bps: u16,
    pub sample_size: u64,
    pub evidence_root: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificationSuccessEstimate {
    pub gold_probability_bps: u16,
    pub novelty_clearance_probability_bps: u16,
    pub sample_size: u64,
    pub evidence_root: String,
}

/// Audited protocol estimate used for routing. Providers may make marketing
/// claims, but those claims are not an input to quality-adjusted selection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtocolSuccessEstimates {
    pub estimator_id: String,
    pub policy_id: PolicyId,
    pub estimated_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub service_estimates: Vec<ServiceSuccessEstimate>,
    pub certification: CertificationSuccessEstimate,
    pub signature: String,
}

impl ProtocolSuccessEstimates {
    pub fn signing_bytes(&self) -> Result<Vec<u8>, CurveError> {
        #[derive(Serialize)]
        struct SigningMaterial<'a> {
            domain: &'static str,
            estimates: &'a ProtocolSuccessEstimates,
        }

        let mut value = serde_json::to_value(SigningMaterial {
            domain: "xlemma-protocol-success-estimates-v1",
            estimates: self,
        })
        .map_err(|_| CurveError::InvalidInput)?;
        value
            .as_object_mut()
            .and_then(|object| object.get_mut("estimates"))
            .and_then(serde_json::Value::as_object_mut)
            .ok_or(CurveError::InvalidInput)?
            .remove("signature");
        xlemma_core::canonical_json_bytes(&value).map_err(|_| CurveError::InvalidInput)
    }

    pub fn verify_signature(&self) -> Result<(), CurveError> {
        xlemma_xlmp::verify_ed25519_detached(
            &self.estimator_id,
            &self.signature,
            &self.signing_bytes()?,
        )
        .map_err(|_| CurveError::InvalidEstimateSignature)
    }
}

/// Deployment policy boundary for estimator authorization. The record's
/// Ed25519 signature proves key control; this policy decides whether that key
/// is trusted for the referenced calibration policy.
pub trait ProtocolEstimatorTrust {
    fn authorizes(&self, estimator_id: &str, policy_id: &PolicyId) -> bool;
}

impl<F> ProtocolEstimatorTrust for F
where
    F: Fn(&str, &PolicyId) -> bool,
{
    fn authorizes(&self, estimator_id: &str, policy_id: &PolicyId) -> bool {
        self(estimator_id, policy_id)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExpectedWork {
    pub units: BTreeMap<ResearchService, u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofCostQuote {
    pub direct_cost: Amount,
    pub service_adjusted_expected_cost: Amount,
    pub risk_adjusted_expected_cost: Amount,
    pub quality_adjusted_certification_cost: Amount,
    pub gold_success_probability_bps: u16,
    pub novelty_clearance_probability_bps: u16,
    pub risk_premium_bps: u16,
    pub selected_offer_ids: Vec<String>,
    pub success_estimate_roots: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchLeadInputs {
    pub contracted_value_units: f64,
    pub reserved_compute_cost_units: f64,
    pub contract_survival_probability: f64,
    pub gold_success_probability: f64,
    pub novelty_clearance_probability: f64,
}

#[derive(Debug, Error)]
pub enum CurveError {
    #[error("no valid offer for required service {0:?}")]
    MissingOffer(ResearchService),
    #[error("offers use incompatible settlement assets")]
    IncompatibleAssets,
    #[error("invalid probability, capacity, scale, or premium")]
    InvalidInput,
    #[error("protocol success estimate has an invalid cryptographic signature")]
    InvalidEstimateSignature,
    #[error("success estimator is not authorized under the calibration policy")]
    UnauthorizedEstimator,
    #[error("cost overflow")]
    Overflow,
}

/// Selects the lowest quality-adjusted offer for each required service.
///
/// This reference selector uses price divided by the independently calibrated
/// protocol completion estimate. Production routing should also model
/// correlation, provider concentration, latency, checker family, privacy, and
/// reserved capacity.
pub fn quote_quality_adjusted_certification_cost(
    now: DateTime<Utc>,
    deadline: DateTime<Utc>,
    work: &ExpectedWork,
    offers: &[ServiceOffer],
    estimates: &ProtocolSuccessEstimates,
    estimator_trust: &impl ProtocolEstimatorTrust,
    risk_premium_bps: u16,
) -> Result<ProofCostQuote, CurveError> {
    estimates.verify_signature()?;
    if !estimator_trust.authorizes(&estimates.estimator_id, &estimates.policy_id) {
        return Err(CurveError::UnauthorizedEstimator);
    }
    let gold_success_probability_bps = estimates.certification.gold_probability_bps;
    let novelty_clearance_probability_bps =
        estimates.certification.novelty_clearance_probability_bps;
    if deadline <= now
        || estimates.estimated_at > now
        || estimates.valid_until <= now
        || estimates.estimator_id.trim().is_empty()
        || estimates.policy_id.validate().is_err()
        || estimates.certification.sample_size == 0
        || estimates.certification.evidence_root.trim().is_empty()
        || !valid_nonzero_probability_bps(gold_success_probability_bps)
        || !valid_nonzero_probability_bps(novelty_clearance_probability_bps)
        || risk_premium_bps > 10_000
    {
        return Err(CurveError::InvalidInput);
    }

    let mut offer_ids = BTreeSet::new();
    for offer in offers {
        if !offer_ids.insert(offer.offer_id.as_str())
            || offer.offer_id.trim().is_empty()
            || offer.provider_id.trim().is_empty()
            || offer.collateral_reference.trim().is_empty()
            || offer.quote_asset.trim().is_empty()
            || offer.quote_decimals > 38
            || offer.quantity_scale == 0
            || offer.capacity_units == 0
            || offer.delivery_start > offer.delivery_end
        {
            return Err(CurveError::InvalidInput);
        }
    }

    let mut estimates_by_offer = BTreeMap::new();
    for estimate in &estimates.service_estimates {
        if estimate.offer_id.trim().is_empty()
            || !valid_nonzero_probability_bps(estimate.completion_probability_bps)
            || estimate.sample_size == 0
            || estimate.evidence_root.trim().is_empty()
            || estimates_by_offer
                .insert(estimate.offer_id.as_str(), estimate)
                .is_some()
        {
            return Err(CurveError::InvalidInput);
        }
    }

    let mut selected = Vec::new();
    let mut estimate_roots = BTreeSet::from([estimates.certification.evidence_root.clone()]);
    let mut asset: Option<(String, u8)> = None;
    let mut direct_cost = 0u128;
    let mut service_adjusted_cost = 0u128;
    let mut warnings = Vec::new();

    for (service, required_units) in &work.units {
        if *required_units == 0 {
            continue;
        }

        let mut best: Option<(&ServiceOffer, &ServiceSuccessEstimate, u128, u128)> = None;
        for offer in offers {
            let Some(estimate) = estimates_by_offer.get(offer.offer_id.as_str()) else {
                continue;
            };
            if offer.service != *service
                || offer.expires_at <= now
                || offer.delivery_start > offer.delivery_end
                || offer.delivery_end <= now
                || offer.delivery_start > deadline
                || offer.delivery_end > deadline
                || offer.quantity_scale == 0
                || offer.capacity_units < *required_units
            {
                continue;
            }

            let raw_cost = ceil_div(
                u128::from(*required_units)
                    .checked_mul(offer.price_minor_units)
                    .ok_or(CurveError::Overflow)?,
                u128::from(offer.quantity_scale),
            )?;
            let adjusted_cost = ceil_div(
                raw_cost.checked_mul(10_000).ok_or(CurveError::Overflow)?,
                u128::from(estimate.completion_probability_bps),
            )?;
            let replace = best
                .as_ref()
                .is_none_or(|(current, _, _, current_adjusted)| {
                    adjusted_cost < *current_adjusted
                        || (adjusted_cost == *current_adjusted && offer.offer_id < current.offer_id)
                });
            if replace {
                best = Some((offer, estimate, raw_cost, adjusted_cost));
            }
        }
        let (best, estimate, raw_cost, adjusted_cost) =
            best.ok_or(CurveError::MissingOffer(*service))?;

        match &asset {
            None => asset = Some((best.quote_asset.clone(), best.quote_decimals)),
            Some((symbol, decimals))
                if *symbol == best.quote_asset && *decimals == best.quote_decimals => {}
            Some(_) => return Err(CurveError::IncompatibleAssets),
        }

        direct_cost = direct_cost
            .checked_add(raw_cost)
            .ok_or(CurveError::Overflow)?;
        service_adjusted_cost = service_adjusted_cost
            .checked_add(adjusted_cost)
            .ok_or(CurveError::Overflow)?;
        selected.push(best.offer_id.clone());
        let _ = estimate_roots.insert(estimate.evidence_root.clone());

        if estimate.completion_probability_bps < 9_000 {
            warnings.push(format!(
                "selected {service:?} offer {} has protocol-estimated completion probability {} bps",
                best.offer_id, estimate.completion_probability_bps
            ));
        }
    }

    let (asset, decimals) = asset.ok_or(CurveError::InvalidInput)?;
    let risk_numerator = service_adjusted_cost
        .checked_mul(10_000u128 + u128::from(risk_premium_bps))
        .ok_or(CurveError::Overflow)?;
    let risk_cost = ceil_div(risk_numerator, 10_000)?;
    let success_joint_bps_squared = u128::from(gold_success_probability_bps)
        .checked_mul(u128::from(novelty_clearance_probability_bps))
        .ok_or(CurveError::Overflow)?;
    let verified_cost = ceil_div(
        risk_cost
            .checked_mul(100_000_000)
            .ok_or(CurveError::Overflow)?,
        success_joint_bps_squared,
    )?;

    Ok(ProofCostQuote {
        direct_cost: Amount::new(direct_cost, asset.clone(), decimals),
        service_adjusted_expected_cost: Amount::new(service_adjusted_cost, asset.clone(), decimals),
        risk_adjusted_expected_cost: Amount::new(risk_cost, asset.clone(), decimals),
        quality_adjusted_certification_cost: Amount::new(verified_cost, asset, decimals),
        gold_success_probability_bps,
        novelty_clearance_probability_bps,
        risk_premium_bps,
        selected_offer_ids: selected,
        success_estimate_roots: estimate_roots.into_iter().collect(),
        warnings,
    })
}

pub fn migration_spread(numerator: &ProofCostQuote, denominator: &ProofCostQuote) -> Option<f64> {
    if numerator.quality_adjusted_certification_cost.asset
        != denominator.quality_adjusted_certification_cost.asset
        || numerator.quality_adjusted_certification_cost.decimals
            != denominator.quality_adjusted_certification_cost.decimals
        || denominator.quality_adjusted_certification_cost.units == 0
    {
        return None;
    }
    Some(
        numerator.quality_adjusted_certification_cost.units as f64
            / denominator.quality_adjusted_certification_cost.units as f64,
    )
}

pub fn research_lead_signal(inputs: &ResearchLeadInputs) -> Result<f64, CurveError> {
    for probability in [
        inputs.contract_survival_probability,
        inputs.gold_success_probability,
        inputs.novelty_clearance_probability,
    ] {
        validate_probability(probability)?;
    }
    if !inputs.contracted_value_units.is_finite()
        || !inputs.reserved_compute_cost_units.is_finite()
        || inputs.contracted_value_units < 0.0
        || inputs.reserved_compute_cost_units <= 0.0
    {
        return Err(CurveError::InvalidInput);
    }

    Ok(
        (inputs.contracted_value_units / inputs.reserved_compute_cost_units)
            * inputs.contract_survival_probability
            * inputs.gold_success_probability
            * inputs.novelty_clearance_probability,
    )
}

fn valid_nonzero_probability_bps(value: u16) -> bool {
    (1..=10_000).contains(&value)
}

fn ceil_div(numerator: u128, denominator: u128) -> Result<u128, CurveError> {
    if denominator == 0 {
        return Err(CurveError::InvalidInput);
    }
    numerator
        .checked_add(denominator - 1)
        .ok_or(CurveError::Overflow)
        .map(|value| value / denominator)
}

fn validate_probability(value: f64) -> Result<(), CurveError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(CurveError::InvalidInput)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use chrono::Duration;
    use ed25519_dalek::{Signer, SigningKey};

    fn offer(
        id: &str,
        service: ResearchService,
        price_minor_units: u128,
        now: DateTime<Utc>,
    ) -> ServiceOffer {
        ServiceOffer {
            offer_id: id.into(),
            provider_id: format!("provider-{id}"),
            service,
            model_or_checker: None,
            hardware_class: None,
            domain: None,
            delivery_start: now,
            delivery_end: now + Duration::hours(1),
            price_minor_units,
            quantity_scale: 100,
            capacity_units: 10_000,
            p95_latency_ms: 1_000,
            collateral_reference: "bond:test".into(),
            quote_asset: "USDC".into(),
            quote_decimals: 6,
            expires_at: now + Duration::days(1),
        }
    }

    fn estimates(now: DateTime<Utc>, values: &[(&str, u16)]) -> ProtocolSuccessEstimates {
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let mut estimates = ProtocolSuccessEstimates {
            estimator_id: format!(
                "ed25519:{}",
                URL_SAFE_NO_PAD.encode(signing_key.verifying_key().as_bytes())
            ),
            policy_id: PolicyId::derive(&"calibration-policy").unwrap(),
            estimated_at: now,
            valid_until: now + Duration::days(1),
            service_estimates: values
                .iter()
                .map(|(offer_id, probability_bps)| ServiceSuccessEstimate {
                    offer_id: (*offer_id).into(),
                    completion_probability_bps: *probability_bps,
                    sample_size: 100,
                    evidence_root: format!("blake3:calibration-{offer_id}"),
                })
                .collect(),
            certification: CertificationSuccessEstimate {
                gold_probability_bps: 8_000,
                novelty_clearance_probability_bps: 5_000,
                sample_size: 100,
                evidence_root: "blake3:job-calibration".into(),
            },
            signature: String::new(),
        };
        estimates.signature = format!(
            "ed25519:{}",
            URL_SAFE_NO_PAD.encode(
                signing_key
                    .sign(&estimates.signing_bytes().unwrap())
                    .to_bytes()
            )
        );
        estimates
    }

    #[test]
    fn quote_uses_quality_adjusted_cost_and_joint_success() {
        let now = Utc::now();
        let mut units = BTreeMap::new();
        let _ = units.insert(ResearchService::AstraGeneration, 200);
        let _ = units.insert(ResearchService::OfficialKernelCheck, 100);
        let work = ExpectedWork { units };
        let offers = vec![
            offer(
                "cheap-unreliable",
                ResearchService::AstraGeneration,
                100,
                now,
            ),
            offer("reliable", ResearchService::AstraGeneration, 200, now),
            offer("kernel", ResearchService::OfficialKernelCheck, 50, now),
        ];
        let estimates = estimates(
            now,
            &[
                ("cheap-unreliable", 2_500),
                ("reliable", 10_000),
                ("kernel", 10_000),
            ],
        );

        let quote = quote_quality_adjusted_certification_cost(
            now,
            now + Duration::hours(2),
            &work,
            &offers,
            &estimates,
            &|signer: &str, policy: &PolicyId| {
                signer == estimates.estimator_id && policy == &estimates.policy_id
            },
            0,
        )
        .unwrap();
        assert!(quote.selected_offer_ids.iter().any(|id| id == "reliable"));
        assert_eq!(quote.direct_cost.units, 450);
        assert_eq!(quote.service_adjusted_expected_cost.units, 450);
        assert_eq!(quote.quality_adjusted_certification_cost.units, 1_125);
    }

    #[test]
    fn incompatible_assets_fail_closed() {
        let now = Utc::now();
        let mut units = BTreeMap::new();
        let _ = units.insert(ResearchService::AstraGeneration, 100);
        let _ = units.insert(ResearchService::Storage, 100);
        let mut storage = offer("storage", ResearchService::Storage, 100, now);
        storage.quote_asset = "DAI".into();
        let offers = vec![
            offer("astra", ResearchService::AstraGeneration, 100, now),
            storage,
        ];
        let estimates = estimates(now, &[("astra", 10_000), ("storage", 10_000)]);
        assert!(matches!(
            quote_quality_adjusted_certification_cost(
                now,
                now + Duration::hours(2),
                &ExpectedWork { units },
                &offers,
                &estimates,
                &|signer: &str, policy: &PolicyId| {
                    signer == estimates.estimator_id && policy == &estimates.policy_id
                },
                0,
            ),
            Err(CurveError::IncompatibleAssets)
        ));
    }

    #[test]
    fn provider_offer_without_protocol_estimate_cannot_enter_routing() {
        let now = Utc::now();
        let mut units = BTreeMap::new();
        let _ = units.insert(ResearchService::AstraGeneration, 100);
        let offers = vec![offer(
            "self-promoted",
            ResearchService::AstraGeneration,
            1,
            now,
        )];
        let estimates = estimates(now, &[]);
        assert!(matches!(
            quote_quality_adjusted_certification_cost(
                now,
                now + Duration::hours(2),
                &ExpectedWork { units },
                &offers,
                &estimates,
                &|signer: &str, policy: &PolicyId| {
                    signer == estimates.estimator_id && policy == &estimates.policy_id
                },
                0,
            ),
            Err(CurveError::MissingOffer(ResearchService::AstraGeneration))
        ));
    }

    #[test]
    fn equal_adjusted_cost_uses_stable_offer_id_tie_break() {
        let now = Utc::now();
        let mut units = BTreeMap::new();
        let _ = units.insert(ResearchService::AstraGeneration, 100);
        let offers = vec![
            offer("z-offer", ResearchService::AstraGeneration, 100, now),
            offer("a-offer", ResearchService::AstraGeneration, 100, now),
        ];
        let estimates = estimates(now, &[("z-offer", 10_000), ("a-offer", 10_000)]);
        let quote = quote_quality_adjusted_certification_cost(
            now,
            now + Duration::hours(2),
            &ExpectedWork { units },
            &offers,
            &estimates,
            &|signer: &str, policy: &PolicyId| {
                signer == estimates.estimator_id && policy == &estimates.policy_id
            },
            0,
        )
        .unwrap();
        assert_eq!(quote.selected_offer_ids, vec!["a-offer".to_owned()]);
    }

    #[test]
    fn provider_cannot_mutate_a_signed_protocol_probability() {
        let now = Utc::now();
        let mut units = BTreeMap::new();
        let _ = units.insert(ResearchService::AstraGeneration, 100);
        let offers = vec![offer(
            "provider",
            ResearchService::AstraGeneration,
            100,
            now,
        )];
        let mut estimates = estimates(now, &[("provider", 8_000)]);
        estimates.service_estimates[0].completion_probability_bps = 10_000;
        assert!(matches!(
            quote_quality_adjusted_certification_cost(
                now,
                now + Duration::hours(2),
                &ExpectedWork { units },
                &offers,
                &estimates,
                &|signer: &str, policy: &PolicyId| {
                    signer == estimates.estimator_id && policy == &estimates.policy_id
                },
                0,
            ),
            Err(CurveError::InvalidEstimateSignature)
        ));
    }

    #[test]
    fn self_signed_but_unauthorized_estimator_is_rejected() {
        let now = Utc::now();
        let mut units = BTreeMap::new();
        let _ = units.insert(ResearchService::AstraGeneration, 100);
        let offers = vec![offer(
            "provider",
            ResearchService::AstraGeneration,
            100,
            now,
        )];
        let estimates = estimates(now, &[("provider", 8_000)]);

        assert!(matches!(
            quote_quality_adjusted_certification_cost(
                now,
                now + Duration::hours(2),
                &ExpectedWork { units },
                &offers,
                &estimates,
                &|_: &str, _: &PolicyId| false,
                0,
            ),
            Err(CurveError::UnauthorizedEstimator)
        ));
    }

    #[test]
    fn published_success_estimate_vector_has_a_valid_signature() {
        let estimates: ProtocolSuccessEstimates = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/protocol-success-estimates.json"
        ))
        .unwrap();
        assert!(estimates.verify_signature().is_ok());
    }
}
