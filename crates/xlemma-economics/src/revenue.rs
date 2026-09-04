use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xlemma_core::{
    Amount, ContributionManifest, MoneyError, ResearcherId, RevenueWaterfall,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueInputs {
    pub gross_collected: Amount,
    pub service_cost: Amount,
    pub compute_cost: Amount,
    pub refunds: Amount,
    pub reserves: Amount,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueAllocation {
    pub net_distributable: Amount,
    pub creator_pool: Amount,
    pub upstream_dependency_pool: Amount,
    pub reverification_security_pool: Amount,
    pub open_research_pool: Amount,
    pub dispute_insurance_pool: Amount,
    pub protocol_operations: Amount,
    pub contributor_allocations: BTreeMap<ResearcherId, Amount>,
    pub contributor_credit_compound: BTreeMap<ResearcherId, Amount>,
    pub contributor_cash_payout: BTreeMap<ResearcherId, Amount>,
    /// Rounding left inside the creator pool after contributor-level splits.
    pub creator_pool_remainder: Amount,
    /// Rounding left after the top-level waterfall split.
    pub rounding_remainder: Amount,
}

#[derive(Debug, Error)]
pub enum RevenueError {
    #[error("revenue waterfall must total exactly 10,000 basis points")]
    InvalidWaterfall,
    #[error("contributor shares must total exactly 10,000 basis points")]
    InvalidContributorShares,
    #[error("auto-compound rate must be at most 10,000 basis points")]
    InvalidCompoundRate,
    #[error("contribution manifest contains a duplicate researcher entry")]
    DuplicateContributor,
    #[error("costs and reserves exceed gross revenue")]
    NegativeNetRevenue,
    #[error(transparent)]
    Money(#[from] MoneyError),
}

pub fn allocate_revenue(
    inputs: &RevenueInputs,
    waterfall: &RevenueWaterfall,
    contributions: &ContributionManifest,
    auto_compound_bps: &BTreeMap<ResearcherId, u16>,
) -> Result<RevenueAllocation, RevenueError> {
    if waterfall.total_bps() != 10_000 {
        return Err(RevenueError::InvalidWaterfall);
    }
    let contribution_total: u32 = contributions
        .contributors
        .iter()
        .map(|contribution| u32::from(contribution.share_bps))
        .sum();
    if contribution_total != 10_000 {
        return Err(RevenueError::InvalidContributorShares);
    }
    let mut unique_contributors = BTreeSet::new();
    if contributions
        .contributors
        .iter()
        .any(|contribution| !unique_contributors.insert(contribution.contributor.clone()))
    {
        return Err(RevenueError::DuplicateContributor);
    }
    if auto_compound_bps.values().any(|rate| *rate > 10_000) {
        return Err(RevenueError::InvalidCompoundRate);
    }

    let mut net = inputs.gross_collected.clone();
    for deduction in [
        &inputs.service_cost,
        &inputs.compute_cost,
        &inputs.refunds,
        &inputs.reserves,
    ] {
        net = net
            .checked_sub(deduction)
            .map_err(|_| RevenueError::NegativeNetRevenue)?;
    }

    let creator_pool = net.mul_bps(waterfall.creator_pool_bps)?;
    let upstream_dependency_pool = net.mul_bps(waterfall.upstream_dependency_pool_bps)?;
    let reverification_security_pool = net.mul_bps(waterfall.reverification_security_pool_bps)?;
    let open_research_pool = net.mul_bps(waterfall.open_research_pool_bps)?;
    let dispute_insurance_pool = net.mul_bps(waterfall.dispute_insurance_pool_bps)?;
    let protocol_operations = net.mul_bps(waterfall.protocol_operations_bps)?;

    let mut contributor_allocations = BTreeMap::new();
    let mut contributor_credit_compound = BTreeMap::new();
    let mut contributor_cash_payout = BTreeMap::new();

    for contribution in &contributions.contributors {
        let allocation = creator_pool.mul_bps(contribution.share_bps)?;
        let compound_rate = auto_compound_bps
            .get(&contribution.contributor)
            .copied()
            .unwrap_or(0);
        let compound = allocation.mul_bps(compound_rate)?;
        let cash = allocation.checked_sub(&compound)?;
        let previous_allocation =
            contributor_allocations.insert(contribution.contributor.clone(), allocation);
        let previous_compound =
            contributor_credit_compound.insert(contribution.contributor.clone(), compound);
        let previous_cash =
            contributor_cash_payout.insert(contribution.contributor.clone(), cash);
        debug_assert!(previous_allocation.is_none());
        debug_assert!(previous_compound.is_none());
        debug_assert!(previous_cash.is_none());
    }

    let contributor_assigned_units: u128 = contributor_allocations
        .values()
        .map(|amount| amount.units)
        .sum();
    let creator_pool_remainder_units = creator_pool
        .units
        .saturating_sub(contributor_assigned_units);

    let allocated_units: u128 = [
        &creator_pool,
        &upstream_dependency_pool,
        &reverification_security_pool,
        &open_research_pool,
        &dispute_insurance_pool,
        &protocol_operations,
    ]
    .iter()
    .map(|amount| amount.units)
    .sum();
    let remainder_units = net.units.saturating_sub(allocated_units);

    Ok(RevenueAllocation {
        net_distributable: net.clone(),
        creator_pool,
        upstream_dependency_pool,
        reverification_security_pool,
        open_research_pool,
        dispute_insurance_pool,
        protocol_operations,
        contributor_allocations,
        contributor_credit_compound,
        contributor_cash_payout,
        creator_pool_remainder: Amount::new(
            creator_pool_remainder_units,
            net.asset.clone(),
            net.decimals,
        ),
        rounding_remainder: Amount::new(remainder_units, net.asset, net.decimals),
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use xlemma_core::{ClaimId, ContributionRole, ContributionShare};

    fn contributions() -> ContributionManifest {
        let claim_id = ClaimId::derive(&"claim").unwrap();
        ContributionManifest {
            claim_id,
            contributors: vec![
                ContributionShare {
                    contributor: ResearcherId::derive(&"a").unwrap(),
                    roles: vec![ContributionRole::FormulaOriginator],
                    share_bps: 6_000,
                    evidence_root: "evidence-a".into(),
                    signed_at: Utc::now(),
                    signature: "signature-a".into(),
                },
                ContributionShare {
                    contributor: ResearcherId::derive(&"b").unwrap(),
                    roles: vec![ContributionRole::LeanFormalizer],
                    share_bps: 4_000,
                    evidence_root: "evidence-b".into(),
                    signed_at: Utc::now(),
                    signature: "signature-b".into(),
                },
            ],
            machine_contributions: vec![],
            amendment_parent: None,
            dispute_status: "clear".into(),
        }
    }

    #[test]
    fn revenue_is_conserved_across_waterfall_and_creator_rounding() {
        let amount = |units| Amount::new(units, "USDC", 6);
        let inputs = RevenueInputs {
            gross_collected: amount(1_003),
            service_cost: amount(1),
            compute_cost: amount(1),
            refunds: amount(0),
            reserves: amount(1),
        };
        let waterfall = RevenueWaterfall {
            creator_pool_bps: 6_500,
            upstream_dependency_pool_bps: 1_000,
            reverification_security_pool_bps: 800,
            open_research_pool_bps: 700,
            dispute_insurance_pool_bps: 500,
            protocol_operations_bps: 500,
        };
        let result = allocate_revenue(&inputs, &waterfall, &contributions(), &BTreeMap::new())
            .unwrap();
        let top_level = result.creator_pool.units
            + result.upstream_dependency_pool.units
            + result.reverification_security_pool.units
            + result.open_research_pool.units
            + result.dispute_insurance_pool.units
            + result.protocol_operations.units
            + result.rounding_remainder.units;
        assert_eq!(top_level, result.net_distributable.units);
        let creator = result.contributor_allocations.values().map(|a| a.units).sum::<u128>()
            + result.creator_pool_remainder.units;
        assert_eq!(creator, result.creator_pool.units);
    }
}
