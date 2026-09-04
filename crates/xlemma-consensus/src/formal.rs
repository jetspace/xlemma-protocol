use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xlemma_core::{
    CheckerFamily, FormalStatus, JobId, ObservationReceipt, ObservationVerdict, OperatorClusterId,
    OperatorId, VerifiedUserId,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormalConsensusPolicy {
    pub required_family_counts: BTreeMap<CheckerFamily, usize>,
    pub minimum_verified_users: usize,
    pub minimum_operators: usize,
    pub minimum_operator_clusters: usize,
    pub minimum_infrastructure_providers: usize,
    pub minimum_regions: usize,
    pub require_identical_artifact_root: bool,
    pub require_identical_environment_root: bool,
    pub require_identical_dependency_root: bool,
    pub require_identical_axiom_set_root: bool,
    pub challenge_period_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormalConsensusOutcome {
    pub status: FormalStatus,
    pub job_id: JobId,
    pub artifact_root: Option<String>,
    pub environment_root: Option<String>,
    pub dependency_root: Option<String>,
    pub axiom_set_root: Option<String>,
    pub pass_count: usize,
    pub fail_count: usize,
    pub error_count: usize,
    pub abstain_count: usize,
    pub verified_users: usize,
    pub operators: usize,
    pub operator_clusters: usize,
    pub infrastructure_providers: usize,
    pub regions: usize,
    pub checker_families: BTreeMap<CheckerFamily, usize>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Error)]
pub enum FormalConsensusError {
    #[error("no observation receipts supplied")]
    Empty,
    #[error("receipts refer to different verification jobs")]
    MixedJobs,
    #[error("duplicate node receipt detected")]
    DuplicateNode,
    #[error("an operator cluster supplied more than one formal observation")]
    DuplicateOperatorCluster,
    #[error("an OperatorID supplied more than one formal observation")]
    DuplicateOperator,
    #[error("a VerifiedUserID supplied more than one formal observation")]
    DuplicateVerifiedUser,
    #[error("missing checker family on a formal observation")]
    MissingCheckerFamily,
    #[error("receipt reveal time precedes its commitment time")]
    InvalidTiming,
    #[error("receipt contains malformed identity or an empty credential-chain root")]
    InvalidIdentity,
    #[error("formal consensus policy is invalid: {0}")]
    InvalidPolicy(String),
}

/// Evaluates already authenticated and committee-authorized observation
/// receipts. Signature, committee-membership, and commit-reveal checks MUST be
/// completed before this pure evidence-sufficiency function is called.
pub fn evaluate_formal_consensus(
    policy: &FormalConsensusPolicy,
    receipts: &[ObservationReceipt],
) -> Result<FormalConsensusOutcome, FormalConsensusError> {
    if receipts.is_empty() {
        return Err(FormalConsensusError::Empty);
    }
    validate_policy(policy)?;

    let job_id = receipts[0].job_id.clone();
    let mut nodes = BTreeSet::new();
    let mut verified_users: BTreeSet<VerifiedUserId> = BTreeSet::new();
    let mut operators: BTreeSet<OperatorId> = BTreeSet::new();
    let mut operator_clusters: BTreeSet<OperatorClusterId> = BTreeSet::new();
    let mut providers = BTreeSet::new();
    let mut regions = BTreeSet::new();
    let mut family_counts = BTreeMap::new();
    let mut reasons = Vec::new();

    for receipt in receipts {
        if receipt.job_id != job_id {
            return Err(FormalConsensusError::MixedJobs);
        }
        if receipt.revealed_at < receipt.committed_at {
            return Err(FormalConsensusError::InvalidTiming);
        }
        if receipt.node_id.validate().is_err()
            || receipt.verified_user_id.validate().is_err()
            || receipt.operator_id.validate().is_err()
            || receipt.operator_cluster_id.validate().is_err()
            || receipt.user_credential_id.validate().is_err()
            || receipt.operator_credential_id.validate().is_err()
            || receipt.node_credential_id.validate().is_err()
            || receipt.credential_chain_root.trim().is_empty()
        {
            return Err(FormalConsensusError::InvalidIdentity);
        }
        if !nodes.insert(receipt.node_id.clone()) {
            return Err(FormalConsensusError::DuplicateNode);
        }
        if !verified_users.insert(receipt.verified_user_id.clone()) {
            return Err(FormalConsensusError::DuplicateVerifiedUser);
        }
        if !operators.insert(receipt.operator_id.clone()) {
            return Err(FormalConsensusError::DuplicateOperator);
        }
        if !operator_clusters.insert(receipt.operator_cluster_id.clone()) {
            return Err(FormalConsensusError::DuplicateOperatorCluster);
        }
        let _ = providers.insert(receipt.infrastructure_provider.as_str());
        let _ = regions.insert(receipt.region.as_str());
        let family = receipt
            .checker_family
            .ok_or(FormalConsensusError::MissingCheckerFamily)?;
        *family_counts.entry(family).or_insert(0usize) += 1;
    }

    let artifact_matches = all_equal(receipts, |receipt| receipt.artifact_root.as_str());
    let environment_matches = all_equal(receipts, |receipt| receipt.environment_root.as_str());
    let dependency_matches = all_equal(receipts, |receipt| receipt.dependency_root.as_str());
    let axiom_matches = all_equal(receipts, |receipt| receipt.axiom_set_root.as_str());

    if policy.require_identical_artifact_root && !artifact_matches {
        reasons.push("artifact roots differ".to_owned());
    }
    if policy.require_identical_environment_root && !environment_matches {
        reasons.push("environment roots differ".to_owned());
    }
    if policy.require_identical_dependency_root && !dependency_matches {
        reasons.push("dependency roots differ".to_owned());
    }
    if policy.require_identical_axiom_set_root && !axiom_matches {
        reasons.push("axiom-set roots differ".to_owned());
    }

    for (family, required) in &policy.required_family_counts {
        let found = family_counts.get(family).copied().unwrap_or(0);
        if found < *required {
            reasons.push(format!(
                "checker family {family:?} requires {required}, found {found}"
            ));
        }
    }

    if verified_users.len() < policy.minimum_verified_users {
        reasons.push(format!(
            "requires {} verified participants, found {}",
            policy.minimum_verified_users,
            verified_users.len()
        ));
    }
    if operators.len() < policy.minimum_operators {
        reasons.push(format!(
            "requires {} independent OperatorIDs, found {}",
            policy.minimum_operators,
            operators.len()
        ));
    }
    if operator_clusters.len() < policy.minimum_operator_clusters {
        reasons.push(format!(
            "requires {} independent operator clusters, found {}",
            policy.minimum_operator_clusters,
            operator_clusters.len()
        ));
    }
    if providers.len() < policy.minimum_infrastructure_providers {
        reasons.push(format!(
            "requires {} infrastructure providers, found {}",
            policy.minimum_infrastructure_providers,
            providers.len()
        ));
    }
    if regions.len() < policy.minimum_regions {
        reasons.push(format!(
            "requires {} regions, found {}",
            policy.minimum_regions,
            regions.len()
        ));
    }

    let pass_count = receipts
        .iter()
        .filter(|receipt| receipt.verdict == ObservationVerdict::Pass)
        .count();
    let fail_count = receipts
        .iter()
        .filter(|receipt| receipt.verdict == ObservationVerdict::Fail)
        .count();
    let error_count = receipts
        .iter()
        .filter(|receipt| receipt.verdict == ObservationVerdict::Error)
        .count();
    let abstain_count = receipts
        .iter()
        .filter(|receipt| receipt.verdict == ObservationVerdict::Abstain)
        .count();

    let required_root_divergence = (policy.require_identical_artifact_root && !artifact_matches)
        || (policy.require_identical_environment_root && !environment_matches)
        || (policy.require_identical_dependency_root && !dependency_matches)
        || (policy.require_identical_axiom_set_root && !axiom_matches);

    let required_evidence_complete = reasons.is_empty();
    let status =
        if required_root_divergence || error_count > 0 || (pass_count > 0 && fail_count > 0) {
            FormalStatus::Divergent
        } else if !required_evidence_complete || abstain_count > 0 {
            FormalStatus::Unchecked
        } else if pass_count == receipts.len() {
            FormalStatus::Certified
        } else if fail_count == receipts.len() {
            FormalStatus::Rejected
        } else {
            FormalStatus::Divergent
        };

    Ok(FormalConsensusOutcome {
        status,
        job_id,
        artifact_root: artifact_matches.then_some(receipts[0].artifact_root.clone()),
        environment_root: environment_matches.then_some(receipts[0].environment_root.clone()),
        dependency_root: dependency_matches.then_some(receipts[0].dependency_root.clone()),
        axiom_set_root: axiom_matches.then_some(receipts[0].axiom_set_root.clone()),
        pass_count,
        fail_count,
        error_count,
        abstain_count,
        verified_users: verified_users.len(),
        operators: operators.len(),
        operator_clusters: operator_clusters.len(),
        infrastructure_providers: providers.len(),
        regions: regions.len(),
        checker_families: family_counts,
        reasons,
    })
}

fn validate_policy(policy: &FormalConsensusPolicy) -> Result<(), FormalConsensusError> {
    if policy.minimum_verified_users == 0
        || policy.minimum_operators == 0
        || policy.minimum_operator_clusters == 0
        || policy.minimum_infrastructure_providers == 0
        || policy.minimum_regions == 0
        || policy
            .required_family_counts
            .values()
            .all(|count| *count == 0)
    {
        return Err(FormalConsensusError::InvalidPolicy(
            "non-zero checker and independence requirements are mandatory".to_owned(),
        ));
    }
    Ok(())
}

fn all_equal<'a, F>(receipts: &'a [ObservationReceipt], selector: F) -> bool
where
    F: Fn(&'a ObservationReceipt) -> &'a str,
{
    receipts.iter().map(selector).collect::<BTreeSet<_>>().len() == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use xlemma_core::{
        NodeCredentialId, NodeId, OperatorCredentialId, ReceiptId, UserCredentialId,
    };

    fn receipt(
        index: usize,
        family: CheckerFamily,
        verdict: ObservationVerdict,
    ) -> ObservationReceipt {
        ObservationReceipt {
            receipt_id: ReceiptId::derive(&format!("receipt-{index}")).unwrap(),
            job_id: JobId::derive(&"job").unwrap(),
            node_id: NodeId::derive(&format!("node-{index}")).unwrap(),
            verified_user_id: VerifiedUserId::derive(&format!("user-{index}")).unwrap(),
            operator_id: OperatorId::derive(&format!("operator-id-{index}")).unwrap(),
            operator_cluster_id: OperatorClusterId::derive(&format!("operator-{index}")).unwrap(),
            user_credential_id: UserCredentialId::derive(&format!("user-credential-{index}"))
                .unwrap(),
            operator_credential_id: OperatorCredentialId::derive(&format!(
                "operator-credential-{index}"
            ))
            .unwrap(),
            node_credential_id: NodeCredentialId::derive(&format!("node-credential-{index}"))
                .unwrap(),
            credential_chain_root: format!("blake3:credential-chain-{index}"),
            checker_family: Some(family),
            checker_name: format!("checker-{index}"),
            checker_version: "1.0.0".into(),
            checker_binary_digest: format!("sha256:checker-{index}"),
            infrastructure_provider: if index == 1 {
                "provider-a"
            } else {
                "provider-b"
            }
            .into(),
            region: if index == 3 { "region-b" } else { "region-a" }.into(),
            artifact_root: "artifact".into(),
            environment_root: "environment".into(),
            dependency_root: "dependencies".into(),
            axiom_set_root: "axioms".into(),
            execution_trace_root: format!("trace-{index}"),
            observation_root: format!("observation-{index}"),
            verdict,
            commitment: "commitment".into(),
            reveal_salt: "salt".into(),
            committed_at: Utc::now(),
            revealed_at: Utc::now(),
            signature: "signature".into(),
        }
    }

    fn policy() -> FormalConsensusPolicy {
        FormalConsensusPolicy {
            required_family_counts: BTreeMap::from([
                (CheckerFamily::LeanKernel, 2),
                (CheckerFamily::Nanoda, 1),
            ]),
            minimum_verified_users: 3,
            minimum_operators: 3,
            minimum_operator_clusters: 3,
            minimum_infrastructure_providers: 2,
            minimum_regions: 2,
            require_identical_artifact_root: true,
            require_identical_environment_root: true,
            require_identical_dependency_root: true,
            require_identical_axiom_set_root: true,
            challenge_period_seconds: 86_400,
        }
    }

    #[test]
    fn gold_quorum_certifies_only_unanimous_exact_reproduction() {
        let receipts = vec![
            receipt(1, CheckerFamily::LeanKernel, ObservationVerdict::Pass),
            receipt(2, CheckerFamily::LeanKernel, ObservationVerdict::Pass),
            receipt(3, CheckerFamily::Nanoda, ObservationVerdict::Pass),
        ];
        let result = evaluate_formal_consensus(&policy(), &receipts).unwrap();
        assert_eq!(result.status, FormalStatus::Certified);
        assert_eq!(result.infrastructure_providers, 2);
        assert_eq!(result.regions, 2);
    }

    #[test]
    fn checker_disagreement_is_divergent_not_majority_vote() {
        let receipts = vec![
            receipt(1, CheckerFamily::LeanKernel, ObservationVerdict::Pass),
            receipt(2, CheckerFamily::LeanKernel, ObservationVerdict::Pass),
            receipt(3, CheckerFamily::Nanoda, ObservationVerdict::Fail),
        ];
        let result = evaluate_formal_consensus(&policy(), &receipts).unwrap();
        assert_eq!(result.status, FormalStatus::Divergent);
    }

    #[test]
    fn correlated_infrastructure_cannot_satisfy_gold_policy() {
        let mut receipts = vec![
            receipt(1, CheckerFamily::LeanKernel, ObservationVerdict::Pass),
            receipt(2, CheckerFamily::LeanKernel, ObservationVerdict::Pass),
            receipt(3, CheckerFamily::Nanoda, ObservationVerdict::Pass),
        ];
        for receipt in &mut receipts {
            receipt.infrastructure_provider = "same-provider".into();
            receipt.region = "same-region".into();
        }
        let result = evaluate_formal_consensus(&policy(), &receipts).unwrap();
        assert_eq!(result.status, FormalStatus::Unchecked);
    }

    #[test]
    fn multiple_nodes_under_one_verified_user_are_not_independent_observations() {
        let mut receipts = vec![
            receipt(1, CheckerFamily::LeanKernel, ObservationVerdict::Pass),
            receipt(2, CheckerFamily::LeanKernel, ObservationVerdict::Pass),
        ];
        receipts[1].verified_user_id = receipts[0].verified_user_id.clone();
        assert!(matches!(
            evaluate_formal_consensus(&policy(), &receipts),
            Err(FormalConsensusError::DuplicateVerifiedUser)
        ));
    }
}
