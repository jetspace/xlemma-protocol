//! Shared node assignment, role-conflict, and observation workflow logic.
//!
//! Model, checker, storage, payment, and chain processes implement adapters
//! around this fail-closed state machine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;
use xlemma_consensus::observation_commitment;
use xlemma_core::{
    ArtifactId, CheckerFamily, ClaimId, JobId, NodeId, NodeRole, ObservationReceipt,
    ObservationVerdict, OperatorClusterId, PolicyId, ReceiptId, TheoryId,
};

/// Domain-separated assignment envelope used by schedulers and node agents.
pub type SignedNodeAssignment = xlemma_crypto::SignedEnvelope<NodeAssignment>;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeCapability {
    pub node_id: NodeId,
    pub operator_cluster_id: OperatorClusterId,
    pub roles: BTreeSet<NodeRole>,
    pub checker_family: Option<CheckerFamily>,
    pub checker_name: Option<String>,
    pub checker_version: Option<String>,
    pub checker_binary_digest: Option<String>,
    pub infrastructure_provider: String,
    pub region: String,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeAssignment {
    pub assignment_id: String,
    pub job_id: JobId,
    pub assigned_node_id: NodeId,
    pub assigned_operator_cluster_id: OperatorClusterId,
    pub role: NodeRole,
    pub claim_id: ClaimId,
    pub theory_id: TheoryId,
    pub artifact_id: ArtifactId,
    pub artifact_root: String,
    pub environment_root: String,
    pub dependency_root: String,
    pub axiom_set_root: String,
    pub policy_id: PolicyId,
    pub assigned_at: DateTime<Utc>,
    pub execute_after: DateTime<Utc>,
    pub commit_deadline: DateTime<Utc>,
    pub reveal_deadline: DateTime<Utc>,
    pub assignment_signature: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeJobState {
    Offered,
    Accepted,
    Executing,
    ObservationReady,
    Committed,
    Revealed,
    Declined,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeJob {
    pub assignment: NodeAssignment,
    pub state: NodeJobState,
    pub accepted_at: Option<DateTime<Utc>>,
    pub commitment: Option<String>,
    pub receipt_id: Option<ReceiptId>,
}

#[derive(Clone, Debug)]
pub struct ObservationEvidence {
    pub verdict: ObservationVerdict,
    pub observation_root: String,
    pub execution_trace_root: String,
}

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("node is inactive")]
    Inactive,
    #[error("node is not qualified for the assigned role")]
    MissingRole,
    #[error("assignment is addressed to a different node or operator cluster")]
    WrongAssignee,
    #[error("checker assignment requires complete checker identity")]
    MissingCheckerIdentity,
    #[error("node capability requires infrastructure-provider and region identity")]
    MissingInfrastructureIdentity,
    #[error("assignment timing is invalid or expired")]
    InvalidTiming,
    #[error("assignment identifier, signature, root, or policy is empty")]
    InvalidAssignment,
    #[error("operator occupies incompatible roles on the same job")]
    RoleConflict,
    #[error("invalid node job transition: {0:?} -> {1:?}")]
    InvalidTransition(NodeJobState, NodeJobState),
    #[error("observation can be committed only after independent execution")]
    ObservationNotReady,
    #[error("reveal does not match the stored commitment")]
    CommitmentMismatch,
    #[error(transparent)]
    Id(#[from] xlemma_core::IdError),
}

pub fn validate_assignment(
    capability: &NodeCapability,
    assignment: &NodeAssignment,
    now: DateTime<Utc>,
    operator_roles_on_job: &BTreeSet<NodeRole>,
) -> Result<(), NodeError> {
    if !capability.active {
        return Err(NodeError::Inactive);
    }
    if !capability.roles.contains(&assignment.role) {
        return Err(NodeError::MissingRole);
    }
    if capability.node_id != assignment.assigned_node_id
        || capability.operator_cluster_id != assignment.assigned_operator_cluster_id
    {
        return Err(NodeError::WrongAssignee);
    }
    if assignment.assignment_id.is_empty()
        || assignment.assignment_signature.is_empty()
        || assignment.artifact_root.is_empty()
        || assignment.environment_root.is_empty()
        || assignment.dependency_root.is_empty()
        || assignment.axiom_set_root.is_empty()
    {
        return Err(NodeError::InvalidAssignment);
    }
    if assignment.assigned_at > now
        || assignment.assigned_at > assignment.execute_after
        || assignment.execute_after > assignment.commit_deadline
        || assignment.commit_deadline >= assignment.reveal_deadline
        || now > assignment.commit_deadline
    {
        return Err(NodeError::InvalidTiming);
    }
    if capability.infrastructure_provider.trim().is_empty()
        || capability.region.trim().is_empty()
    {
        return Err(NodeError::MissingInfrastructureIdentity);
    }
    if is_checker_role(assignment.role)
        && (capability.checker_family.is_none()
            || capability.checker_name.as_deref().unwrap_or_default().is_empty()
            || capability
                .checker_version
                .as_deref()
                .unwrap_or_default()
                .is_empty()
            || capability
                .checker_binary_digest
                .as_deref()
                .unwrap_or_default()
                .is_empty())
    {
        return Err(NodeError::MissingCheckerIdentity);
    }
    for existing in operator_roles_on_job {
        if roles_conflict(*existing, assignment.role) {
            return Err(NodeError::RoleConflict);
        }
    }
    Ok(())
}

pub fn roles_conflict(left: NodeRole, right: NodeRole) -> bool {
    use NodeRole::*;
    if left == right {
        return is_checker_role(left);
    }
    matches!(
        (left, right),
        (Researcher, OfficialKernelChecker | IndependentChecker | NoveltyReviewer | CertificateFinalizer)
            | (OfficialKernelChecker | IndependentChecker | NoveltyReviewer | CertificateFinalizer, Researcher)
            | (AstraProver, OfficialKernelChecker | IndependentChecker | CertificateFinalizer)
            | (OfficialKernelChecker | IndependentChecker | CertificateFinalizer, AstraProver)
            | (LeanBuilder, OfficialKernelChecker | IndependentChecker)
            | (OfficialKernelChecker | IndependentChecker, LeanBuilder)
            | (PaymentFacilitator, CertificateFinalizer)
            | (CertificateFinalizer, PaymentFacilitator)
            | (Challenger, CertificateFinalizer)
            | (CertificateFinalizer, Challenger)
    )
}

pub fn transition(job: &mut NodeJob, next: NodeJobState) -> Result<(), NodeError> {
    use NodeJobState::*;
    let valid = matches!(
        (job.state, next),
        (Offered, Accepted)
            | (Offered, Declined)
            | (Offered, Expired)
            | (Accepted, Executing)
            | (Accepted, Expired)
            | (Executing, ObservationReady)
            | (Executing, Expired)
            | (ObservationReady, Committed)
            | (Committed, Revealed)
            | (Committed, Expired)
    );
    if !valid {
        return Err(NodeError::InvalidTransition(job.state, next));
    }
    job.state = next;
    Ok(())
}

pub fn build_observation_receipt(
    capability: &NodeCapability,
    assignment: &NodeAssignment,
    evidence: ObservationEvidence,
    salt: &str,
    committed_at: DateTime<Utc>,
    revealed_at: DateTime<Utc>,
    signature: String,
) -> Result<ObservationReceipt, NodeError> {
    if salt.is_empty()
        || signature.is_empty()
        || committed_at < assignment.execute_after
        || committed_at > assignment.commit_deadline
        || revealed_at < committed_at
        || revealed_at > assignment.reveal_deadline
    {
        return Err(NodeError::InvalidTiming);
    }
    if capability.node_id != assignment.assigned_node_id
        || capability.operator_cluster_id != assignment.assigned_operator_cluster_id
    {
        return Err(NodeError::WrongAssignee);
    }
    let family = capability.checker_family;
    if is_checker_role(assignment.role) && family.is_none() {
        return Err(NodeError::MissingCheckerIdentity);
    }
    let commitment = observation_commitment(
        &assignment.job_id,
        evidence.verdict,
        &evidence.observation_root,
        salt.as_bytes(),
    );
    let receipt_material = serde_json::json!({
        "job_id": &assignment.job_id,
        "node_id": &capability.node_id,
        "operator_cluster_id": &capability.operator_cluster_id,
        "checker_family": family,
        "checker_name": &capability.checker_name,
        "checker_version": &capability.checker_version,
        "checker_binary_digest": &capability.checker_binary_digest,
        "infrastructure_provider": &capability.infrastructure_provider,
        "region": &capability.region,
        "artifact_root": &assignment.artifact_root,
        "environment_root": &assignment.environment_root,
        "dependency_root": &assignment.dependency_root,
        "axiom_set_root": &assignment.axiom_set_root,
        "execution_trace_root": &evidence.execution_trace_root,
        "observation_root": &evidence.observation_root,
        "verdict": evidence.verdict,
        "commitment": &commitment,
        "committed_at": &committed_at,
        "revealed_at": &revealed_at,
    });
    Ok(ObservationReceipt {
        receipt_id: ReceiptId::derive(&receipt_material)?,
        job_id: assignment.job_id.clone(),
        node_id: capability.node_id.clone(),
        operator_cluster_id: capability.operator_cluster_id.clone(),
        checker_family: family,
        checker_name: capability.checker_name.clone().unwrap_or_default(),
        checker_version: capability.checker_version.clone().unwrap_or_default(),
        checker_binary_digest: capability
            .checker_binary_digest
            .clone()
            .unwrap_or_default(),
        infrastructure_provider: capability.infrastructure_provider.clone(),
        region: capability.region.clone(),
        artifact_root: assignment.artifact_root.clone(),
        environment_root: assignment.environment_root.clone(),
        dependency_root: assignment.dependency_root.clone(),
        axiom_set_root: assignment.axiom_set_root.clone(),
        execution_trace_root: evidence.execution_trace_root,
        observation_root: evidence.observation_root,
        verdict: evidence.verdict,
        commitment,
        reveal_salt: salt.to_owned(),
        committed_at,
        revealed_at,
        signature,
    })
}

fn is_checker_role(role: NodeRole) -> bool {
    matches!(
        role,
        NodeRole::OfficialKernelChecker | NodeRole::IndependentChecker
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use xlemma_consensus::verify_reveal;

    fn capability(role: NodeRole) -> NodeCapability {
        NodeCapability {
            node_id: NodeId::derive(&"node").unwrap(),
            operator_cluster_id: OperatorClusterId::derive(&"operator").unwrap(),
            roles: BTreeSet::from([role]),
            checker_family: Some(CheckerFamily::LeanKernel),
            checker_name: Some("lean4checker".into()),
            checker_version: Some("4.33.1".into()),
            checker_binary_digest: Some("sha256:checker".into()),
            infrastructure_provider: "provider-a".into(),
            region: "region-a".into(),
            active: true,
        }
    }

    fn assignment(role: NodeRole, now: DateTime<Utc>) -> NodeAssignment {
        NodeAssignment {
            assignment_id: "assignment-1".into(),
            job_id: JobId::derive(&"job").unwrap(),
            assigned_node_id: NodeId::derive(&"node").unwrap(),
            assigned_operator_cluster_id: OperatorClusterId::derive(&"operator").unwrap(),
            role,
            claim_id: ClaimId::derive(&"claim").unwrap(),
            theory_id: TheoryId::derive(&"theory").unwrap(),
            artifact_id: ArtifactId::derive(&"artifact").unwrap(),
            artifact_root: "blake3:artifact".into(),
            environment_root: "blake3:environment".into(),
            dependency_root: "blake3:dependencies".into(),
            axiom_set_root: "blake3:axioms".into(),
            policy_id: PolicyId::derive(&"policy").unwrap(),
            assigned_at: now - Duration::seconds(1),
            execute_after: now - Duration::seconds(1),
            commit_deadline: now + Duration::minutes(5),
            reveal_deadline: now + Duration::minutes(10),
            assignment_signature: "signature".into(),
        }
    }

    #[test]
    fn prover_cannot_also_verify_its_own_job() {
        assert!(roles_conflict(
            NodeRole::AstraProver,
            NodeRole::OfficialKernelChecker
        ));
    }

    #[test]
    fn assignment_checks_operator_role_conflicts() {
        let now = Utc::now();
        let cap = capability(NodeRole::OfficialKernelChecker);
        let roles = BTreeSet::from([NodeRole::AstraProver]);
        assert!(matches!(
            validate_assignment(
                &cap,
                &assignment(NodeRole::OfficialKernelChecker, now),
                now,
                &roles,
            ),
            Err(NodeError::RoleConflict)
        ));
    }

    #[test]
    fn generated_receipt_has_a_verifiable_commitment() {
        let now = Utc::now();
        let cap = capability(NodeRole::OfficialKernelChecker);
        let assignment = assignment(NodeRole::OfficialKernelChecker, now);
        let receipt = build_observation_receipt(
            &cap,
            &assignment,
            ObservationEvidence {
                verdict: ObservationVerdict::Pass,
                observation_root: "blake3:observation".into(),
                execution_trace_root: "blake3:trace".into(),
            },
            "fresh-salt",
            now,
            now + Duration::seconds(1),
            "signature".into(),
        )
        .unwrap();
        assert!(verify_reveal(&receipt, b"fresh-salt"));
    }

    #[test]
    fn assignment_is_bound_to_the_selected_node() {
        let now = Utc::now();
        let cap = capability(NodeRole::OfficialKernelChecker);
        let mut assigned = assignment(NodeRole::OfficialKernelChecker, now);
        assigned.assigned_node_id = NodeId::derive(&"another-node").unwrap();
        assert!(matches!(
            validate_assignment(&cap, &assigned, now, &BTreeSet::new()),
            Err(NodeError::WrongAssignee)
        ));
    }


    #[test]
    fn assignment_rejects_future_issue_time() {
        let now = Utc::now();
        let cap = capability(NodeRole::OfficialKernelChecker);
        let mut assigned = assignment(NodeRole::OfficialKernelChecker, now);
        assigned.assigned_at = now + Duration::seconds(1);
        assert!(matches!(
            validate_assignment(&cap, &assigned, now, &BTreeSet::new()),
            Err(NodeError::InvalidTiming)
        ));
    }

    #[test]
    fn checker_requires_provider_and_region_identity() {
        let now = Utc::now();
        let mut cap = capability(NodeRole::OfficialKernelChecker);
        cap.infrastructure_provider.clear();
        assert!(matches!(
            validate_assignment(
                &cap,
                &assignment(NodeRole::OfficialKernelChecker, now),
                now,
                &BTreeSet::new(),
            ),
            Err(NodeError::MissingInfrastructureIdentity)
        ));
    }

    #[test]
    fn receipt_cannot_be_revealed_after_the_assignment_deadline() {
        let now = Utc::now();
        let cap = capability(NodeRole::OfficialKernelChecker);
        let assigned = assignment(NodeRole::OfficialKernelChecker, now);
        let result = build_observation_receipt(
            &cap,
            &assigned,
            ObservationEvidence {
                verdict: ObservationVerdict::Pass,
                observation_root: "blake3:observation".into(),
                execution_trace_root: "blake3:trace".into(),
            },
            "fresh-salt",
            now,
            assigned.reveal_deadline + Duration::seconds(1),
            "signature".into(),
        );
        assert!(matches!(result, Err(NodeError::InvalidTiming)));
    }

}
