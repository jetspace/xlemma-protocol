use serde::{Deserialize, Serialize};
use thiserror::Error;
use xlemma_core::Amount;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComputeSavingsEvidence {
    /// Estimated mean cost without the upstream lemma, in settlement minor units.
    pub counterfactual_without_mean_units: f64,
    pub counterfactual_without_std_error_units: f64,
    pub observed_with_units: u128,
    /// One-sided conservative multiplier, e.g. 1.645 for approximately 95% LCB.
    pub lower_confidence_multiplier: f64,
    pub evidence_sample_size: u64,
    pub upstream_is_in_final_dependency_graph: bool,
    pub equivalent_cluster_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeSavingsPolicy {
    /// Fraction of conservatively established savings paid upstream.
    pub savings_share_bps: u16,
    /// Maximum fraction of downstream net revenue available to this dividend.
    pub downstream_revenue_cap_bps: u16,
    pub minimum_sample_size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeSavingsDividend {
    pub conservative_savings: Amount,
    pub uncapped_dividend: Amount,
    pub revenue_cap: Amount,
    pub payable_dividend: Amount,
}

#[derive(Debug, Error)]
pub enum DividendError {
    #[error("upstream lemma is not in the final proof dependency graph")]
    NotFinalDependency,
    #[error("insufficient counterfactual sample size")]
    InsufficientSample,
    #[error("invalid statistical input")]
    InvalidEvidence,
    #[error("basis-point value exceeds 10,000")]
    InvalidPolicy,
}

pub fn compute_savings_dividend(
    evidence: &ComputeSavingsEvidence,
    policy: &ComputeSavingsPolicy,
    downstream_net_revenue: &Amount,
) -> Result<ComputeSavingsDividend, DividendError> {
    if !evidence.upstream_is_in_final_dependency_graph {
        return Err(DividendError::NotFinalDependency);
    }
    if evidence.evidence_sample_size < policy.minimum_sample_size {
        return Err(DividendError::InsufficientSample);
    }
    if policy.savings_share_bps > 10_000 || policy.downstream_revenue_cap_bps > 10_000 {
        return Err(DividendError::InvalidPolicy);
    }
    if !evidence.counterfactual_without_mean_units.is_finite()
        || !evidence.counterfactual_without_std_error_units.is_finite()
        || !evidence.lower_confidence_multiplier.is_finite()
        || evidence.counterfactual_without_std_error_units < 0.0
        || evidence.lower_confidence_multiplier < 0.0
    {
        return Err(DividendError::InvalidEvidence);
    }

    let lower_bound_without = evidence.counterfactual_without_mean_units
        - evidence.lower_confidence_multiplier * evidence.counterfactual_without_std_error_units;
    let conservative_savings_units =
        (lower_bound_without - evidence.observed_with_units as f64).max(0.0) as u128;

    let uncapped_units = conservative_savings_units
        .saturating_mul(u128::from(policy.savings_share_bps))
        / 10_000;
    let cap_units = downstream_net_revenue
        .units
        .saturating_mul(u128::from(policy.downstream_revenue_cap_bps))
        / 10_000;
    let payable_units = uncapped_units.min(cap_units);

    let amount = |units| {
        Amount::new(
            units,
            downstream_net_revenue.asset.clone(),
            downstream_net_revenue.decimals,
        )
    };

    Ok(ComputeSavingsDividend {
        conservative_savings: amount(conservative_savings_units),
        uncapped_dividend: amount(uncapped_units),
        revenue_cap: amount(cap_units),
        payable_dividend: amount(payable_units),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dividend_uses_lower_bound_and_revenue_cap() {
        let evidence = ComputeSavingsEvidence {
            counterfactual_without_mean_units: 1_000.0,
            counterfactual_without_std_error_units: 100.0,
            observed_with_units: 500,
            lower_confidence_multiplier: 1.0,
            evidence_sample_size: 100,
            upstream_is_in_final_dependency_graph: true,
            equivalent_cluster_id: "cluster-1".into(),
        };
        let policy = ComputeSavingsPolicy {
            savings_share_bps: 5_000,
            downstream_revenue_cap_bps: 1_000,
            minimum_sample_size: 30,
        };
        let revenue = Amount::new(1_000, "USDC", 6);
        let result = compute_savings_dividend(&evidence, &policy, &revenue).unwrap();
        assert_eq!(result.conservative_savings.units, 400);
        assert_eq!(result.uncapped_dividend.units, 200);
        assert_eq!(result.revenue_cap.units, 100);
        assert_eq!(result.payable_dividend.units, 100);
    }

    #[test]
    fn unused_dependency_cannot_receive_dividend() {
        let evidence = ComputeSavingsEvidence {
            counterfactual_without_mean_units: 1_000.0,
            counterfactual_without_std_error_units: 0.0,
            observed_with_units: 100,
            lower_confidence_multiplier: 1.0,
            evidence_sample_size: 100,
            upstream_is_in_final_dependency_graph: false,
            equivalent_cluster_id: "cluster-1".into(),
        };
        assert!(matches!(
            compute_savings_dividend(
                &evidence,
                &ComputeSavingsPolicy {
                    savings_share_bps: 1_000,
                    downstream_revenue_cap_bps: 1_000,
                    minimum_sample_size: 1,
                },
                &Amount::new(1_000, "USDC", 6),
            ),
            Err(DividendError::NotFinalDependency)
        ));
    }
}
