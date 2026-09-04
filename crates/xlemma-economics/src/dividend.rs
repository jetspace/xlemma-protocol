use serde::{Deserialize, Serialize};
use thiserror::Error;
use xlemma_core::{Amount, ReceiptId, RevenueEventId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeSavingsEvidence {
    pub evidence_id: ReceiptId,
    /// Estimated mean cost without the upstream lemma, in settlement minor units.
    pub counterfactual_without_mean_units: u128,
    pub counterfactual_without_std_error_units: u128,
    pub observed_with_units: u128,
    /// One-sided conservative multiplier in basis points; 16,450 means 1.645.
    pub lower_confidence_multiplier_bps: u32,
    pub settlement_asset: String,
    pub settlement_decimals: u8,
    pub evidence_sample_size: u64,
    pub upstream_is_in_final_dependency_graph: bool,
    pub equivalent_cluster_id: String,
}

impl ComputeSavingsEvidence {
    fn identity_value(&self) -> Result<serde_json::Value, ImpactAllocationError> {
        let mut value =
            serde_json::to_value(self).map_err(|_| ImpactAllocationError::InvalidEvidence)?;
        value
            .as_object_mut()
            .ok_or(ImpactAllocationError::InvalidEvidence)?
            .remove("evidence_id");
        Ok(value)
    }

    pub fn expected_evidence_id(&self) -> Result<ReceiptId, ImpactAllocationError> {
        ReceiptId::derive(&self.identity_value()?)
            .map_err(|_| ImpactAllocationError::InvalidEvidence)
    }

    pub fn validate_integrity(&self) -> Result<(), ImpactAllocationError> {
        if self.evidence_id.validate().is_err()
            || self.evidence_id != self.expected_evidence_id()?
            || self.lower_confidence_multiplier_bps == 0
            || self.lower_confidence_multiplier_bps > 100_000
            || self.settlement_asset.trim().is_empty()
            || self.settlement_decimals > 38
            || self.evidence_sample_size == 0
            || self.equivalent_cluster_id.trim().is_empty()
        {
            return Err(ImpactAllocationError::InvalidEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeSavingsPolicy {
    /// Fraction of conservatively established savings paid upstream.
    pub savings_share_bps: u16,
    /// Maximum fraction of downstream net revenue available to this dividend.
    pub downstream_revenue_cap_bps: u16,
    pub minimum_sample_size: u64,
}

/// Prescriptive authorization for one bounded impact-pool allocation. A
/// formal dependency and a compute-savings estimate cannot create this object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactPoolAuthorization {
    pub authorization_id: ReceiptId,
    pub revenue_event_id: RevenueEventId,
    pub compute_savings_evidence_id: ReceiptId,
    pub economic_policy_root: String,
    pub eligible_economic_edge_root: String,
    pub settlement_receipt_id: ReceiptId,
    pub pool_budget: Amount,
    pub non_recursive: bool,
    pub authorizer: String,
    pub signature: String,
}

impl ImpactPoolAuthorization {
    fn identity_value(&self) -> Result<serde_json::Value, ImpactAllocationError> {
        let mut value =
            serde_json::to_value(self).map_err(|_| ImpactAllocationError::InvalidEvidence)?;
        let object = value
            .as_object_mut()
            .ok_or(ImpactAllocationError::InvalidEvidence)?;
        object.remove("authorization_id");
        object.remove("signature");
        Ok(value)
    }

    pub fn expected_authorization_id(&self) -> Result<ReceiptId, ImpactAllocationError> {
        ReceiptId::derive(&self.identity_value()?)
            .map_err(|_| ImpactAllocationError::InvalidEvidence)
    }

    pub fn signing_bytes(&self) -> Result<Vec<u8>, ImpactAllocationError> {
        #[derive(Serialize)]
        struct SigningMaterial<'a> {
            domain: &'static str,
            authorization: &'a ImpactPoolAuthorization,
        }

        let mut value = serde_json::to_value(SigningMaterial {
            domain: "xlemma-impact-pool-authorization-v1",
            authorization: self,
        })
        .map_err(|_| ImpactAllocationError::InvalidEvidence)?;
        value
            .as_object_mut()
            .and_then(|object| object.get_mut("authorization"))
            .and_then(serde_json::Value::as_object_mut)
            .ok_or(ImpactAllocationError::InvalidEvidence)?
            .remove("signature");
        xlemma_core::canonical_json_bytes(&value)
            .map_err(|_| ImpactAllocationError::InvalidEvidence)
    }

    pub fn validate_integrity(&self) -> Result<(), ImpactAllocationError> {
        if self.authorization_id.validate().is_err()
            || self.revenue_event_id.validate().is_err()
            || self.compute_savings_evidence_id.validate().is_err()
            || self.settlement_receipt_id.validate().is_err()
            || self.authorization_id != self.expected_authorization_id()?
            || self.economic_policy_root.trim().is_empty()
            || self.eligible_economic_edge_root.trim().is_empty()
            || self.authorizer.trim().is_empty()
            || self.pool_budget.units == 0
            || self.pool_budget.asset.trim().is_empty()
            || self.pool_budget.decimals > 38
        {
            return Err(ImpactAllocationError::MissingEconomicAuthorization);
        }
        if !self.non_recursive {
            return Err(ImpactAllocationError::RecursiveAllocation);
        }
        xlemma_xlmp::verify_ed25519_detached(
            &self.authorizer,
            &self.signature,
            &self.signing_bytes()?,
        )
        .map_err(|_| ImpactAllocationError::InvalidAuthorizationSignature)
    }
}

/// Deployment policy boundary for impact-pool authorizers. The detached
/// signature proves key control; it does not make an arbitrary signer trusted.
pub trait ImpactAuthorizerTrust {
    fn authorizes(&self, authorizer: &str, economic_policy_root: &str) -> bool;
}

impl<F> ImpactAuthorizerTrust for F
where
    F: Fn(&str, &str) -> bool,
{
    fn authorizes(&self, authorizer: &str, economic_policy_root: &str) -> bool {
        self(authorizer, economic_policy_root)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactPoolAllocation {
    pub conservative_savings: Amount,
    pub uncapped_allocation: Amount,
    pub revenue_cap: Amount,
    pub impact_pool_cap: Amount,
    pub payable_allocation: Amount,
}

#[derive(Debug, Error)]
pub enum ImpactAllocationError {
    #[error("upstream lemma is not in the final proof dependency graph")]
    NotFinalDependency,
    #[error("insufficient counterfactual sample size")]
    InsufficientSample,
    #[error("invalid statistical input")]
    InvalidEvidence,
    #[error("basis-point value exceeds 10,000")]
    InvalidPolicy,
    #[error("compute savings are not authorized by a settled economic policy and edge")]
    MissingEconomicAuthorization,
    #[error("impact-pool authorization has an invalid cryptographic signature")]
    InvalidAuthorizationSignature,
    #[error("impact-pool authorizer is not trusted for the economic policy")]
    UnauthorizedImpactAuthorizer,
    #[error("recursive allocation of the same revenue event is prohibited")]
    RecursiveAllocation,
    #[error("impact-pool budget uses an incompatible asset")]
    IncompatiblePool,
    #[error("compute-savings evidence uses an incompatible asset")]
    IncompatibleEvidenceAsset,
    #[error("impact-pool authorization does not bind the supplied evidence")]
    EvidenceAuthorizationMismatch,
    #[error("checked impact-allocation arithmetic overflowed")]
    ArithmeticOverflow,
}

pub fn compute_impact_pool_allocation(
    evidence: &ComputeSavingsEvidence,
    policy: &ComputeSavingsPolicy,
    downstream_net_revenue: &Amount,
    authorization: &ImpactPoolAuthorization,
    authorizer_trust: &impl ImpactAuthorizerTrust,
) -> Result<ImpactPoolAllocation, ImpactAllocationError> {
    evidence.validate_integrity()?;
    if downstream_net_revenue.asset.trim().is_empty() || downstream_net_revenue.decimals > 38 {
        return Err(ImpactAllocationError::IncompatibleEvidenceAsset);
    }
    if !evidence.upstream_is_in_final_dependency_graph {
        return Err(ImpactAllocationError::NotFinalDependency);
    }
    if evidence.evidence_sample_size < policy.minimum_sample_size {
        return Err(ImpactAllocationError::InsufficientSample);
    }
    if policy.savings_share_bps > 10_000 || policy.downstream_revenue_cap_bps > 10_000 {
        return Err(ImpactAllocationError::InvalidPolicy);
    }
    authorization.validate_integrity()?;
    if authorization.compute_savings_evidence_id != evidence.evidence_id {
        return Err(ImpactAllocationError::EvidenceAuthorizationMismatch);
    }
    if !authorizer_trust.authorizes(
        &authorization.authorizer,
        &authorization.economic_policy_root,
    ) {
        return Err(ImpactAllocationError::UnauthorizedImpactAuthorizer);
    }
    if downstream_net_revenue
        .ensure_compatible(&authorization.pool_budget)
        .is_err()
    {
        return Err(ImpactAllocationError::IncompatiblePool);
    }
    if evidence.settlement_asset != downstream_net_revenue.asset
        || evidence.settlement_decimals != downstream_net_revenue.decimals
    {
        return Err(ImpactAllocationError::IncompatibleEvidenceAsset);
    }

    let confidence_deduction = ceil_div(
        evidence
            .counterfactual_without_std_error_units
            .checked_mul(u128::from(evidence.lower_confidence_multiplier_bps))
            .ok_or(ImpactAllocationError::ArithmeticOverflow)?,
        10_000,
    )?;
    let lower_bound_without = evidence
        .counterfactual_without_mean_units
        .saturating_sub(confidence_deduction);
    let conservative_savings_units =
        lower_bound_without.saturating_sub(evidence.observed_with_units);

    let uncapped_units = conservative_savings_units
        .checked_mul(u128::from(policy.savings_share_bps))
        .ok_or(ImpactAllocationError::ArithmeticOverflow)?
        / 10_000;
    let cap_units = downstream_net_revenue
        .units
        .checked_mul(u128::from(policy.downstream_revenue_cap_bps))
        .ok_or(ImpactAllocationError::ArithmeticOverflow)?
        / 10_000;
    let payable_units = uncapped_units
        .min(cap_units)
        .min(authorization.pool_budget.units);

    let amount = |units| {
        Amount::new(
            units,
            downstream_net_revenue.asset.clone(),
            downstream_net_revenue.decimals,
        )
    };

    Ok(ImpactPoolAllocation {
        conservative_savings: amount(conservative_savings_units),
        uncapped_allocation: amount(uncapped_units),
        revenue_cap: amount(cap_units),
        impact_pool_cap: authorization.pool_budget.clone(),
        payable_allocation: amount(payable_units),
    })
}

fn ceil_div(numerator: u128, denominator: u128) -> Result<u128, ImpactAllocationError> {
    if denominator == 0 {
        return Err(ImpactAllocationError::InvalidEvidence);
    }
    numerator
        .checked_add(denominator - 1)
        .ok_or(ImpactAllocationError::ArithmeticOverflow)
        .map(|value| value / denominator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use ed25519_dalek::{Signer, SigningKey};

    fn evidence(
        mean_units: u128,
        std_error_units: u128,
        observed_units: u128,
        final_dependency: bool,
    ) -> ComputeSavingsEvidence {
        let mut evidence = ComputeSavingsEvidence {
            evidence_id: ReceiptId::derive(&"placeholder-evidence").unwrap(),
            counterfactual_without_mean_units: mean_units,
            counterfactual_without_std_error_units: std_error_units,
            observed_with_units: observed_units,
            lower_confidence_multiplier_bps: 10_000,
            settlement_asset: "USDC".into(),
            settlement_decimals: 6,
            evidence_sample_size: 100,
            upstream_is_in_final_dependency_graph: final_dependency,
            equivalent_cluster_id: "cluster-1".into(),
        };
        evidence.evidence_id = evidence.expected_evidence_id().unwrap();
        evidence
    }

    fn authorization(
        economic_policy_root: &str,
        economic_edge_root: &str,
        evidence_id: ReceiptId,
        pool_units: u128,
    ) -> ImpactPoolAuthorization {
        let signing_key = SigningKey::from_bytes(&[9_u8; 32]);
        let mut authorization = ImpactPoolAuthorization {
            authorization_id: ReceiptId::derive(&"placeholder").unwrap(),
            revenue_event_id: RevenueEventId::derive(&"revenue-event").unwrap(),
            compute_savings_evidence_id: evidence_id,
            economic_policy_root: economic_policy_root.into(),
            eligible_economic_edge_root: economic_edge_root.into(),
            settlement_receipt_id: ReceiptId::derive(&"settlement").unwrap(),
            pool_budget: Amount::new(pool_units, "USDC", 6),
            non_recursive: true,
            authorizer: format!(
                "ed25519:{}",
                URL_SAFE_NO_PAD.encode(signing_key.verifying_key().as_bytes())
            ),
            signature: String::new(),
        };
        authorization.authorization_id = authorization.expected_authorization_id().unwrap();
        authorization.signature = format!(
            "ed25519:{}",
            URL_SAFE_NO_PAD.encode(
                signing_key
                    .sign(&authorization.signing_bytes().unwrap())
                    .to_bytes()
            )
        );
        authorization
    }

    #[test]
    fn impact_allocation_uses_lower_bound_and_revenue_cap() {
        let evidence = evidence(1_000, 100, 500, true);
        let policy = ComputeSavingsPolicy {
            savings_share_bps: 5_000,
            downstream_revenue_cap_bps: 1_000,
            minimum_sample_size: 30,
        };
        let revenue = Amount::new(1_000, "USDC", 6);
        let authorization = authorization(
            "blake3:economic-policy",
            "blake3:economic-edge",
            evidence.evidence_id.clone(),
            80,
        );
        let result = compute_impact_pool_allocation(
            &evidence,
            &policy,
            &revenue,
            &authorization,
            &|authorizer: &str, policy_root: &str| {
                authorizer == authorization.authorizer
                    && policy_root == authorization.economic_policy_root
            },
        )
        .unwrap();
        assert_eq!(result.conservative_savings.units, 400);
        assert_eq!(result.uncapped_allocation.units, 200);
        assert_eq!(result.revenue_cap.units, 100);
        assert_eq!(result.impact_pool_cap.units, 80);
        assert_eq!(result.payable_allocation.units, 80);
    }

    #[test]
    fn unused_dependency_cannot_receive_dividend() {
        let evidence = evidence(1_000, 0, 100, false);
        assert!(matches!(
            compute_impact_pool_allocation(
                &evidence,
                &ComputeSavingsPolicy {
                    savings_share_bps: 1_000,
                    downstream_revenue_cap_bps: 1_000,
                    minimum_sample_size: 1,
                },
                &Amount::new(1_000, "USDC", 6),
                &authorization(
                    "blake3:economic-policy",
                    "blake3:economic-edge",
                    evidence.evidence_id.clone(),
                    100,
                ),
                &|_: &str, _: &str| true,
            ),
            Err(ImpactAllocationError::NotFinalDependency)
        ));
    }

    #[test]
    fn compute_signal_without_economic_authorization_cannot_pay() {
        let evidence = evidence(1_000, 0, 100, true);
        let authorization = authorization("", "", evidence.evidence_id.clone(), 100);
        assert!(matches!(
            compute_impact_pool_allocation(
                &evidence,
                &ComputeSavingsPolicy {
                    savings_share_bps: 1_000,
                    downstream_revenue_cap_bps: 1_000,
                    minimum_sample_size: 1,
                },
                &Amount::new(1_000, "USDC", 6),
                &authorization,
                &|_: &str, _: &str| true,
            ),
            Err(ImpactAllocationError::MissingEconomicAuthorization)
        ));
    }

    #[test]
    fn self_signed_but_unauthorized_impact_allocation_is_rejected() {
        let evidence = evidence(1_000, 0, 100, true);
        let authorization = authorization(
            "blake3:economic-policy",
            "blake3:economic-edge",
            evidence.evidence_id.clone(),
            100,
        );
        assert!(matches!(
            compute_impact_pool_allocation(
                &evidence,
                &ComputeSavingsPolicy {
                    savings_share_bps: 1_000,
                    downstream_revenue_cap_bps: 1_000,
                    minimum_sample_size: 1,
                },
                &Amount::new(1_000, "USDC", 6),
                &authorization,
                &|_: &str, _: &str| false,
            ),
            Err(ImpactAllocationError::UnauthorizedImpactAuthorizer)
        ));
    }

    #[test]
    fn published_impact_authorization_has_content_identity_and_signature() {
        let authorization: ImpactPoolAuthorization = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/impact-pool-authorization.json"
        ))
        .unwrap();
        assert!(authorization.validate_integrity().is_ok());
    }

    #[test]
    fn published_compute_savings_evidence_has_content_identity() {
        let evidence: ComputeSavingsEvidence = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/compute-savings-evidence.json"
        ))
        .unwrap();
        assert!(evidence.validate_integrity().is_ok());
    }

    #[test]
    fn impact_authorization_identity_binds_the_exact_revenue_event() {
        let mut authorization = authorization(
            "blake3:economic-policy",
            "blake3:economic-edge",
            ReceiptId::derive(&"evidence").unwrap(),
            100,
        );
        authorization.revenue_event_id = RevenueEventId::derive(&"another-event").unwrap();
        assert!(matches!(
            authorization.validate_integrity(),
            Err(ImpactAllocationError::MissingEconomicAuthorization)
        ));
    }
}
