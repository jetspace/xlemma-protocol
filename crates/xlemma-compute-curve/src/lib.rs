//! Spot and forward pricing for proof-production and verification services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use xlemma_core::Amount;

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
    pub completion_probability: f64,
    pub p95_latency_ms: u64,
    pub collateral_reference: String,
    pub quote_asset: String,
    pub quote_decimals: u8,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExpectedWork {
    pub units: BTreeMap<ResearchService, f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofCostQuote {
    pub direct_expected_cost: Amount,
    pub risk_adjusted_expected_cost: Amount,
    pub verified_proof_cost: Amount,
    pub gold_success_probability: f64,
    pub novelty_clearance_probability: f64,
    pub risk_premium_bps: u16,
    pub selected_offer_ids: Vec<String>,
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
    #[error("cost overflow")]
    Overflow,
}

/// Selects the lowest quality-adjusted offer for each required service.
///
/// This reference selector uses price / completion probability. Production
/// routing should also model correlation, provider concentration, latency,
/// checker family, privacy, and reserved capacity.
pub fn quote_verified_proof_cost(
    now: DateTime<Utc>,
    deadline: DateTime<Utc>,
    work: &ExpectedWork,
    offers: &[ServiceOffer],
    gold_success_probability: f64,
    novelty_clearance_probability: f64,
    risk_premium_bps: u16,
) -> Result<ProofCostQuote, CurveError> {
    validate_probability(gold_success_probability)?;
    validate_probability(novelty_clearance_probability)?;
    if deadline <= now
        || gold_success_probability <= 0.0
        || novelty_clearance_probability <= 0.0
        || risk_premium_bps > 10_000
    {
        return Err(CurveError::InvalidInput);
    }

    let mut selected = Vec::new();
    let mut asset: Option<(String, u8)> = None;
    let mut direct_cost = 0u128;
    let mut warnings = Vec::new();

    for (service, required_units) in &work.units {
        if !required_units.is_finite() || *required_units < 0.0 {
            return Err(CurveError::InvalidInput);
        }
        if *required_units == 0.0 {
            continue;
        }

        let best = offers
            .iter()
            .filter(|offer| {
                offer.service == *service
                    && offer.expires_at > now
                    && offer.delivery_start <= offer.delivery_end
                    && offer.delivery_end > now
                    && offer.delivery_start <= deadline
                    && offer.delivery_end <= deadline
                    && offer.quantity_scale > 0
                    && offer.capacity_units as f64 >= *required_units
                    && offer.completion_probability > 0.0
                    && offer.completion_probability <= 1.0
            })
            .min_by(|left, right| {
                quality_adjusted_unit_price(left)
                    .partial_cmp(&quality_adjusted_unit_price(right))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or(CurveError::MissingOffer(*service))?;

        match &asset {
            None => asset = Some((best.quote_asset.clone(), best.quote_decimals)),
            Some((symbol, decimals))
                if *symbol == best.quote_asset && *decimals == best.quote_decimals => {}
            Some(_) => return Err(CurveError::IncompatibleAssets),
        }

        let service_cost =
            ((*required_units / best.quantity_scale as f64) * best.price_minor_units as f64).ceil();
        if !service_cost.is_finite() || service_cost < 0.0 || service_cost > u128::MAX as f64 {
            return Err(CurveError::Overflow);
        }
        direct_cost = direct_cost
            .checked_add(service_cost as u128)
            .ok_or(CurveError::Overflow)?;
        selected.push(best.offer_id.clone());

        if best.completion_probability < 0.9 {
            warnings.push(format!(
                "selected {service:?} offer {} has completion probability {:.3}",
                best.offer_id, best.completion_probability
            ));
        }
    }

    let (asset, decimals) = asset.ok_or(CurveError::InvalidInput)?;
    let risk_numerator = direct_cost
        .checked_mul(10_000u128 + u128::from(risk_premium_bps))
        .ok_or(CurveError::Overflow)?;
    let risk_cost = risk_numerator
        .checked_add(9_999)
        .ok_or(CurveError::Overflow)?
        / 10_000;
    let success_joint = gold_success_probability * novelty_clearance_probability;
    let verified_cost = (risk_cost as f64 / success_joint).ceil();
    if verified_cost > u128::MAX as f64 {
        return Err(CurveError::Overflow);
    }

    Ok(ProofCostQuote {
        direct_expected_cost: Amount::new(direct_cost, asset.clone(), decimals),
        risk_adjusted_expected_cost: Amount::new(risk_cost, asset.clone(), decimals),
        verified_proof_cost: Amount::new(verified_cost as u128, asset, decimals),
        gold_success_probability,
        novelty_clearance_probability,
        risk_premium_bps,
        selected_offer_ids: selected,
        warnings,
    })
}

pub fn migration_spread(numerator: &ProofCostQuote, denominator: &ProofCostQuote) -> Option<f64> {
    if numerator.verified_proof_cost.asset != denominator.verified_proof_cost.asset
        || numerator.verified_proof_cost.decimals != denominator.verified_proof_cost.decimals
        || denominator.verified_proof_cost.units == 0
    {
        return None;
    }
    Some(numerator.verified_proof_cost.units as f64 / denominator.verified_proof_cost.units as f64)
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

fn quality_adjusted_unit_price(offer: &ServiceOffer) -> f64 {
    offer.price_minor_units as f64 / offer.quantity_scale as f64 / offer.completion_probability
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
    use chrono::Duration;

    fn offer(
        id: &str,
        service: ResearchService,
        price_minor_units: u128,
        probability: f64,
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
            completion_probability: probability,
            p95_latency_ms: 1_000,
            collateral_reference: "bond:test".into(),
            quote_asset: "USDC".into(),
            quote_decimals: 6,
            expires_at: now + Duration::days(1),
        }
    }

    #[test]
    fn quote_uses_quality_adjusted_cost_and_joint_success() {
        let now = Utc::now();
        let mut units = BTreeMap::new();
        let _ = units.insert(ResearchService::AstraGeneration, 200.0);
        let _ = units.insert(ResearchService::OfficialKernelCheck, 100.0);
        let work = ExpectedWork { units };
        let offers = vec![
            offer(
                "cheap-unreliable",
                ResearchService::AstraGeneration,
                100,
                0.25,
                now,
            ),
            offer("reliable", ResearchService::AstraGeneration, 200, 1.0, now),
            offer("kernel", ResearchService::OfficialKernelCheck, 50, 1.0, now),
        ];

        let quote =
            quote_verified_proof_cost(now, now + Duration::hours(2), &work, &offers, 0.8, 0.5, 0)
                .unwrap();
        assert!(quote.selected_offer_ids.iter().any(|id| id == "reliable"));
        assert_eq!(quote.direct_expected_cost.units, 450);
        assert_eq!(quote.verified_proof_cost.units, 1_125);
    }

    #[test]
    fn incompatible_assets_fail_closed() {
        let now = Utc::now();
        let mut units = BTreeMap::new();
        let _ = units.insert(ResearchService::AstraGeneration, 100.0);
        let _ = units.insert(ResearchService::Storage, 100.0);
        let mut storage = offer("storage", ResearchService::Storage, 100, 1.0, now);
        storage.quote_asset = "DAI".into();
        let offers = vec![
            offer("astra", ResearchService::AstraGeneration, 100, 1.0, now),
            storage,
        ];
        assert!(matches!(
            quote_verified_proof_cost(
                now,
                now + Duration::hours(2),
                &ExpectedWork { units },
                &offers,
                0.8,
                0.8,
                0,
            ),
            Err(CurveError::IncompatibleAssets)
        ));
    }
}
