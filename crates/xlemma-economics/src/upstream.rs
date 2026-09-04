//! Deterministic bounded upstream-pool allocation and revisable knowledge
//! productivity signals.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xlemma_core::{
    Amount, CapsuleEconomicMode, ClaimId, EconomicConstitution, EconomicGraphEdge, MoneyError,
    ReceiptId, RevenueEvent, RevenueEventId,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamCandidate {
    pub edge: EconomicGraphEdge,
    pub used_in_final_artifact: bool,
    pub use_weight_bps: u16,
    pub evidence_quality_bps: u16,
    pub dependency_depth: u16,
    pub equivalence_cluster: String,
    pub use_evidence_root: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamAllocation {
    pub revenue_event_id: RevenueEventId,
    pub downstream_claim_id: ClaimId,
    pub net_qualifying_revenue: Amount,
    pub upstream_pool: Amount,
    pub payouts: BTreeMap<ClaimId, Amount>,
    pub selected_equivalence_clusters: BTreeMap<String, ClaimId>,
    pub unallocated_remainder: Amount,
    pub non_recursive: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeProductivityObservation {
    pub claim_id: ClaimId,
    pub independently_verified_downstream_outcomes: u64,
    pub attributable_effort_milliunits: u64,
    pub confidence_bps: u16,
    pub evidence_root: String,
    pub observed_at: chrono::DateTime<chrono::Utc>,
    pub supersedes: Option<ReceiptId>,
}

impl KnowledgeProductivityObservation {
    /// Returns outcomes per effort unit in millionths. This is an impact and
    /// ranking signal, never a receivable or settlement amount.
    pub fn score_millionths(&self) -> Result<u128, UpstreamError> {
        self.validate()?;
        u128::from(self.independently_verified_downstream_outcomes)
            .checked_mul(1_000_000)
            .ok_or(UpstreamError::Overflow)
            .map(|value| value / u128::from(self.attributable_effort_milliunits))
    }

    pub const fn creates_payment_debt(&self) -> bool {
        false
    }

    pub fn validate(&self) -> Result<(), UpstreamError> {
        self.claim_id.validate()?;
        if let Some(receipt_id) = &self.supersedes {
            receipt_id.validate()?;
        }
        if self.independently_verified_downstream_outcomes == 0
            || self.attributable_effort_milliunits == 0
            || self.confidence_bps == 0
            || self.confidence_bps > 10_000
            || self.evidence_root.trim().is_empty()
        {
            return Err(UpstreamError::InvalidImpactObservation);
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum UpstreamError {
    #[error("economic constitution is invalid or has no allocable upstream pool")]
    InvalidConstitution,
    #[error("revenue event is not qualifying settled downstream revenue")]
    NonQualifyingRevenue,
    #[error("revenue event was already consumed by an upstream pool")]
    RevenueEventAlreadyConsumed,
    #[error("upstream payouts cannot recursively create another upstream pool")]
    RecursiveAllocation,
    #[error("candidate lacks final use, explicit authorization, or bounded graph evidence")]
    IneligibleCandidate,
    #[error("no eligible independent equivalence cluster remains")]
    NoEligibleCandidate,
    #[error("knowledge-productivity observation is incomplete or unbounded")]
    InvalidImpactObservation,
    #[error("checked integer arithmetic overflow")]
    Overflow,
    #[error(transparent)]
    Money(#[from] MoneyError),
    #[error(transparent)]
    Id(#[from] xlemma_core::IdError),
}

/// Allocates exactly one bounded upstream pool for one settled revenue event.
///
/// Equivalent claims compete as one cluster: only the highest-weight member of
/// a cluster remains. This prevents equivalent-statement splitting from
/// manufacturing additional pool weight. Integer division rounds down and the
/// remainder stays unallocated.
pub fn allocate_upstream_pool(
    event: &RevenueEvent,
    constitution: &EconomicConstitution,
    candidates: &[UpstreamCandidate],
    consumed_revenue_events: &BTreeSet<RevenueEventId>,
    event_is_upstream_distribution: bool,
) -> Result<UpstreamAllocation, UpstreamError> {
    event
        .validate_integrity()
        .map_err(|_| UpstreamError::NonQualifyingRevenue)?;
    constitution
        .validate()
        .map_err(|_| UpstreamError::InvalidConstitution)?;
    if event_is_upstream_distribution {
        return Err(UpstreamError::RecursiveAllocation);
    }
    if consumed_revenue_events.contains(&event.revenue_event_id) {
        return Err(UpstreamError::RevenueEventAlreadyConsumed);
    }
    if constitution.mode == CapsuleEconomicMode::Commons
        || constitution.upstream_pool_bps == 0
        || !constitution
            .qualifying_revenue_sources
            .contains(&event.source)
        || event.related_party
        || event.evidence_root.trim().is_empty()
        || event.signature.trim().is_empty()
    {
        return Err(UpstreamError::NonQualifyingRevenue);
    }

    let mut net = event.gross_collected.clone();
    for deduction in [&event.refunds, &event.service_costs, &event.reserves] {
        net = net
            .checked_sub(deduction)
            .map_err(|_| UpstreamError::NonQualifyingRevenue)?;
    }
    if net.units == 0 {
        return Err(UpstreamError::NonQualifyingRevenue);
    }
    let upstream_pool = net.mul_bps(constitution.upstream_pool_bps)?;
    if upstream_pool.units == 0 {
        return Err(UpstreamError::NonQualifyingRevenue);
    }

    #[derive(Clone)]
    struct WeightedCandidate {
        claim_id: ClaimId,
        weight: u128,
        edge_cap_bps: u16,
    }

    let mut clustered: BTreeMap<String, WeightedCandidate> = BTreeMap::new();
    for candidate in candidates {
        candidate
            .edge
            .validate()
            .map_err(|_| UpstreamError::IneligibleCandidate)?;
        if !candidate.used_in_final_artifact
            || candidate.use_weight_bps == 0
            || candidate.use_weight_bps > 10_000
            || candidate.evidence_quality_bps == 0
            || candidate.evidence_quality_bps > 10_000
            || candidate.dependency_depth == 0
            || candidate.dependency_depth > constitution.max_dependency_depth
            || candidate.equivalence_cluster.trim().is_empty()
            || candidate.use_evidence_root.trim().is_empty()
            || candidate.edge.downstream_claim_id != event.claim_id
            || candidate.edge.policy_id != constitution.policy_id
            || candidate.edge.qualifying_revenue_source != event.source
            || event.realized_at < candidate.edge.valid_from
            || event.realized_at >= candidate.edge.valid_until
        {
            return Err(UpstreamError::IneligibleCandidate);
        }
        let mut weight = u128::from(candidate.use_weight_bps)
            .checked_mul(u128::from(candidate.evidence_quality_bps))
            .ok_or(UpstreamError::Overflow)?
            / 10_000;
        for _ in 1..candidate.dependency_depth {
            weight = weight
                .checked_mul(u128::from(constitution.depth_decay_bps))
                .ok_or(UpstreamError::Overflow)?
                / 10_000;
        }
        if weight == 0 {
            continue;
        }
        let weighted = WeightedCandidate {
            claim_id: candidate.edge.upstream_claim_id.clone(),
            weight,
            edge_cap_bps: candidate.edge.cap_bps,
        };
        clustered
            .entry(candidate.equivalence_cluster.clone())
            .and_modify(|current| {
                if weighted.weight > current.weight
                    || (weighted.weight == current.weight && weighted.claim_id < current.claim_id)
                {
                    *current = weighted.clone();
                }
            })
            .or_insert(weighted);
    }
    if clustered.is_empty() {
        return Err(UpstreamError::NoEligibleCandidate);
    }

    let mut ranked = clustered.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .weight
            .cmp(&left.1.weight)
            .then_with(|| left.1.claim_id.cmp(&right.1.claim_id))
    });
    ranked.truncate(usize::from(constitution.max_ancestors));
    let total_weight = ranked.iter().try_fold(0u128, |total, (_, candidate)| {
        total
            .checked_add(candidate.weight)
            .ok_or(UpstreamError::Overflow)
    })?;
    if total_weight == 0 {
        return Err(UpstreamError::NoEligibleCandidate);
    }

    let mut payouts = BTreeMap::new();
    let mut selected_equivalence_clusters = BTreeMap::new();
    let mut allocated_units = 0u128;
    for (cluster, candidate) in ranked {
        let proportional = upstream_pool
            .units
            .checked_mul(candidate.weight)
            .ok_or(UpstreamError::Overflow)?
            / total_weight;
        let cap_bps = candidate
            .edge_cap_bps
            .min(constitution.per_ancestor_cap_bps);
        let cap = net.mul_bps(cap_bps)?;
        let units = proportional.min(cap.units);
        if units < constitution.minimum_payout_units {
            continue;
        }
        allocated_units = allocated_units
            .checked_add(units)
            .ok_or(UpstreamError::Overflow)?;
        if payouts
            .insert(
                candidate.claim_id.clone(),
                Amount::new(units, net.asset.clone(), net.decimals),
            )
            .is_some()
        {
            return Err(UpstreamError::IneligibleCandidate);
        }
        selected_equivalence_clusters.insert(cluster, candidate.claim_id);
    }
    if allocated_units > upstream_pool.units {
        return Err(UpstreamError::Overflow);
    }
    let unallocated_remainder = Amount::new(
        upstream_pool.units - allocated_units,
        net.asset.clone(),
        net.decimals,
    );
    Ok(UpstreamAllocation {
        revenue_event_id: event.revenue_event_id.clone(),
        downstream_claim_id: event.claim_id.clone(),
        net_qualifying_revenue: net,
        upstream_pool,
        payouts,
        selected_equivalence_clusters,
        unallocated_remainder,
        non_recursive: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use xlemma_core::{EconomicEdgeKind, PolicyId, TheoryId};

    fn claim(name: &str) -> ClaimId {
        ClaimId::from_canonical_elaborated_type(&TheoryId::derive(&"theory").unwrap(), name)
            .unwrap()
    }

    fn constitution() -> EconomicConstitution {
        EconomicConstitution {
            policy_id: PolicyId::derive(&"reciprocal").unwrap(),
            mode: CapsuleEconomicMode::Reciprocal,
            qualifying_revenue_sources: BTreeSet::from(["license".into()]),
            upstream_pool_bps: 500,
            per_ancestor_cap_bps: 300,
            max_ancestors: 8,
            max_dependency_depth: 4,
            depth_decay_bps: 5_000,
            minimum_payout_units: 1,
            bare_citations_eligible: false,
            externally_independent_claims_eligible: false,
            downstream_veto: false,
            recursive_charging: false,
            single_charge_per_revenue_event: true,
            equivalent_claim_clustering: true,
            policy_root: "blake3:policy".into(),
            signatures: vec!["signature".into()],
        }
    }

    fn event() -> RevenueEvent {
        let mut event = RevenueEvent {
            revenue_event_id: RevenueEventId::derive(&"placeholder").unwrap(),
            claim_id: claim("downstream"),
            source: "license".into(),
            related_party: false,
            settlement_receipt_id: ReceiptId::derive(&"settlement").unwrap(),
            gross_collected: Amount::new(10_000, "USDC", 6),
            refunds: Amount::new(500, "USDC", 6),
            service_costs: Amount::new(500, "USDC", 6),
            reserves: Amount::new(1_000, "USDC", 6),
            realized_at: Utc::now(),
            evidence_root: "blake3:settlement".into(),
            signature: "signature".into(),
        };
        event.revenue_event_id = event.derive_revenue_event_id().unwrap();
        event
    }

    fn candidate(upstream: &str, cluster: &str, weight: u16) -> UpstreamCandidate {
        let event = event();
        UpstreamCandidate {
            edge: EconomicGraphEdge {
                downstream_claim_id: event.claim_id,
                upstream_claim_id: claim(upstream),
                kind: EconomicEdgeKind::ContributesToUpstreamPool,
                policy_id: constitution().policy_id,
                qualifying_revenue_source: "license".into(),
                cap_bps: 300,
                authorization_root: "blake3:authorization".into(),
                valid_from: event.realized_at - Duration::days(1),
                valid_until: event.realized_at + Duration::days(1),
                signatures: vec!["signature".into()],
            },
            used_in_final_artifact: true,
            use_weight_bps: weight,
            evidence_quality_bps: 10_000,
            dependency_depth: 1,
            equivalence_cluster: cluster.into(),
            use_evidence_root: "blake3:proof-use".into(),
        }
    }

    #[test]
    fn one_pool_is_bounded_clustered_and_conserved() {
        let event = event();
        let result = allocate_upstream_pool(
            &event,
            &constitution(),
            &[
                candidate("equivalent-low", "same-theorem", 1_000),
                candidate("equivalent-high", "same-theorem", 8_000),
                candidate("other", "other-theorem", 2_000),
            ],
            &BTreeSet::new(),
            false,
        )
        .unwrap();
        assert_eq!(result.net_qualifying_revenue.units, 8_000);
        assert_eq!(result.upstream_pool.units, 400);
        assert_eq!(result.selected_equivalence_clusters.len(), 2);
        let allocated: u128 = result.payouts.values().map(|amount| amount.units).sum();
        assert_eq!(allocated + result.unallocated_remainder.units, 400);
        assert!(!result.payouts.contains_key(&claim("equivalent-low")));
    }

    #[test]
    fn revenue_event_cannot_be_charged_twice_or_recursively() {
        let event = event();
        assert!(matches!(
            allocate_upstream_pool(
                &event,
                &constitution(),
                &[candidate("upstream", "cluster", 10_000)],
                &BTreeSet::from([event.revenue_event_id.clone()]),
                false,
            ),
            Err(UpstreamError::RevenueEventAlreadyConsumed)
        ));
        assert!(matches!(
            allocate_upstream_pool(
                &event,
                &constitution(),
                &[candidate("upstream", "cluster", 10_000)],
                &BTreeSet::new(),
                true,
            ),
            Err(UpstreamError::RecursiveAllocation)
        ));
    }

    #[test]
    fn related_party_settlement_cannot_manufacture_upstream_demand() {
        let mut event = event();
        event.related_party = true;
        event.revenue_event_id = event.derive_revenue_event_id().unwrap();
        assert!(matches!(
            allocate_upstream_pool(
                &event,
                &constitution(),
                &[candidate("upstream", "cluster", 10_000)],
                &BTreeSet::new(),
                false,
            ),
            Err(UpstreamError::NonQualifyingRevenue)
        ));
    }

    #[test]
    fn dust_below_the_declared_minimum_stays_unallocated() {
        let event = event();
        let mut policy = constitution();
        policy.minimum_payout_units = 401;
        let result = allocate_upstream_pool(
            &event,
            &policy,
            &[candidate("upstream", "cluster", 10_000)],
            &BTreeSet::new(),
            false,
        )
        .unwrap();
        assert!(result.payouts.is_empty());
        assert_eq!(result.unallocated_remainder, result.upstream_pool);
    }

    #[test]
    fn property_upstream_allocations_are_deterministic_bounded_and_conserved() {
        let gross_values = [1u128, 2, 3, 17, 101, 10_000, 1_000_003, 1_000_000_000];
        let pool_rates = [1u16, 50, 300, 500, 1_000, 5_000, 10_000];
        let weight_sets = [
            [1u16, 1, 1],
            [1, 9_999, 10_000],
            [3_333, 3_333, 3_334],
            [10_000, 5_000, 1_000],
        ];
        for gross in gross_values {
            for pool_bps in pool_rates {
                for weights in weight_sets {
                    let mut event = event();
                    event.gross_collected.units = gross;
                    event.refunds.units = 0;
                    event.service_costs.units = 0;
                    event.reserves.units = 0;
                    event.revenue_event_id = event.derive_revenue_event_id().unwrap();
                    let mut constitution = constitution();
                    constitution.upstream_pool_bps = pool_bps;
                    constitution.per_ancestor_cap_bps = pool_bps;
                    let candidates = [
                        candidate("property-a", "cluster-a", weights[0]),
                        candidate("property-b", "cluster-b", weights[1]),
                        candidate("property-c", "cluster-c", weights[2]),
                    ];
                    let forward = allocate_upstream_pool(
                        &event,
                        &constitution,
                        &candidates,
                        &BTreeSet::new(),
                        false,
                    );
                    if event.gross_collected.mul_bps(pool_bps).unwrap().units == 0 {
                        assert!(matches!(forward, Err(UpstreamError::NonQualifyingRevenue)));
                        continue;
                    }
                    let forward = forward.unwrap();
                    let reversed_candidates = [
                        candidates[2].clone(),
                        candidates[1].clone(),
                        candidates[0].clone(),
                    ];
                    let reversed = allocate_upstream_pool(
                        &event,
                        &constitution,
                        &reversed_candidates,
                        &BTreeSet::new(),
                        false,
                    )
                    .unwrap();
                    assert_eq!(forward, reversed);
                    let allocated = forward
                        .payouts
                        .values()
                        .try_fold(0u128, |total, amount| total.checked_add(amount.units))
                        .unwrap();
                    assert_eq!(
                        allocated + forward.unallocated_remainder.units,
                        forward.upstream_pool.units
                    );
                    assert!(forward.upstream_pool.units <= gross);
                    assert!(forward.payouts.len() <= usize::from(constitution.max_ancestors));
                }
            }
        }
    }

    #[test]
    fn knowledge_productivity_is_revisable_signal_not_debt() {
        let observation = KnowledgeProductivityObservation {
            claim_id: claim("productive"),
            independently_verified_downstream_outcomes: 4,
            attributable_effort_milliunits: 2_000,
            confidence_bps: 7_500,
            evidence_root: "blake3:impact".into(),
            observed_at: Utc::now(),
            supersedes: Some(ReceiptId::derive(&"prior").unwrap()),
        };
        assert_eq!(observation.score_millionths().unwrap(), 2_000);
        assert!(!observation.creates_payment_debt());
    }
}
