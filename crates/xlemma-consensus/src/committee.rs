use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xlemma_core::{NodeId, NodeRole, OperatorClusterId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EligibleNode {
    pub node_id: NodeId,
    pub operator_cluster_id: OperatorClusterId,
    pub roles: BTreeSet<NodeRole>,
    pub collateral_units: u128,
    pub reliability_bps: u16,
    pub qualification_bps: u16,
    pub infrastructure_provider: Option<String>,
    pub region: Option<String>,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitteeRequirement {
    pub role: NodeRole,
    pub count: usize,
    pub minimum_collateral_units: u128,
    pub minimum_reliability_bps: u16,
    pub minimum_qualification_bps: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitteeSelection {
    pub seed_commitment: String,
    pub selected: BTreeMap<NodeRole, Vec<NodeId>>,
}

#[derive(Debug, Error)]
pub enum CommitteeError {
    #[error("not enough eligible independent operators for role {role:?}: need {needed}, found {found}")]
    InsufficientEligibleNodes {
        role: NodeRole,
        needed: usize,
        found: usize,
    },
}

/// Deterministic, stake-capped sortition reference algorithm.
///
/// Stake is an eligibility bond, not unbounded voting weight. Once a node
/// meets the requirement, selection is hash-ranked using public randomness.
/// Production deployments SHOULD source the seed from a manipulation-resistant
/// VRF or equivalent beacon and prove the selection on chain.
pub fn select_committee(
    seed: &[u8],
    nodes: &[EligibleNode],
    requirements: &[CommitteeRequirement],
) -> Result<CommitteeSelection, CommitteeError> {
    let mut globally_used_operators = BTreeSet::new();
    let mut selected = BTreeMap::new();

    for requirement in requirements {
        let mut ranked: Vec<([u8; 32], &EligibleNode)> = nodes
            .iter()
            .filter(|node| {
                node.active
                    && node.roles.contains(&requirement.role)
                    && node.collateral_units >= requirement.minimum_collateral_units
                    && node.reliability_bps >= requirement.minimum_reliability_bps
                    && node.qualification_bps >= requirement.minimum_qualification_bps
                    && !globally_used_operators.contains(&node.operator_cluster_id)
            })
            .map(|node| {
                let mut hasher = blake3::Hasher::new();
                hasher.update(b"xlemma-committee-v1\0");
                hasher.update(seed);
                hasher.update(b"\0");
                hasher.update(format!("{:?}", requirement.role).as_bytes());
                hasher.update(b"\0");
                hasher.update(node.node_id.as_str().as_bytes());
                (*hasher.finalize().as_bytes(), node)
            })
            .collect();

        ranked.sort_by_key(|(rank, _)| *rank);

        if ranked.len() < requirement.count {
            return Err(CommitteeError::InsufficientEligibleNodes {
                role: requirement.role,
                needed: requirement.count,
                found: ranked.len(),
            });
        }

        let chosen: Vec<_> = ranked
            .into_iter()
            .take(requirement.count)
            .map(|(_, node)| {
                let inserted = globally_used_operators.insert(node.operator_cluster_id.clone());
                debug_assert!(inserted);
                node.node_id.clone()
            })
            .collect();
        let previous = selected.insert(requirement.role, chosen);
        debug_assert!(previous.is_none());
    }

    let seed_commitment = format!("blake3:{}", blake3::hash(seed).to_hex());
    Ok(CommitteeSelection {
        seed_commitment,
        selected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eligible_node(
        label: &str,
        operator: &str,
        roles: BTreeSet<NodeRole>,
    ) -> EligibleNode {
        EligibleNode {
            node_id: NodeId::derive(&label).unwrap(),
            operator_cluster_id: OperatorClusterId::derive(&operator).unwrap(),
            roles,
            collateral_units: 1_000,
            reliability_bps: 9_900,
            qualification_bps: 9_500,
            infrastructure_provider: Some(format!("provider-{label}")),
            region: Some(format!("region-{label}")),
            active: true,
        }
    }

    #[test]
    fn selection_is_deterministic_and_operator_independent() {
        let role = NodeRole::OfficialKernelChecker;
        let nodes = vec![
            eligible_node("a", "op-a", BTreeSet::from([role])),
            eligible_node("b", "op-b", BTreeSet::from([role])),
            eligible_node("c", "op-c", BTreeSet::from([role])),
        ];
        let requirements = vec![CommitteeRequirement {
            role,
            count: 2,
            minimum_collateral_units: 100,
            minimum_reliability_bps: 9_000,
            minimum_qualification_bps: 9_000,
        }];

        let left = select_committee(b"public-seed", &nodes, &requirements).unwrap();
        let right = select_committee(b"public-seed", &nodes, &requirements).unwrap();
        assert_eq!(left.seed_commitment, right.seed_commitment);
        assert_eq!(left.selected, right.selected);
        assert_eq!(left.selected[&role].len(), 2);
    }

    #[test]
    fn one_operator_cannot_fill_multiple_required_roles() {
        let kernel = NodeRole::OfficialKernelChecker;
        let independent = NodeRole::IndependentChecker;
        let both = BTreeSet::from([kernel, independent]);
        let nodes = vec![eligible_node("a", "same-operator", both)];
        let requirements = vec![
            CommitteeRequirement {
                role: kernel,
                count: 1,
                minimum_collateral_units: 1,
                minimum_reliability_bps: 1,
                minimum_qualification_bps: 1,
            },
            CommitteeRequirement {
                role: independent,
                count: 1,
                minimum_collateral_units: 1,
                minimum_reliability_bps: 1,
                minimum_qualification_bps: 1,
            },
        ];

        assert!(matches!(
            select_committee(b"seed", &nodes, &requirements),
            Err(CommitteeError::InsufficientEligibleNodes { .. })
        ));
    }
}
