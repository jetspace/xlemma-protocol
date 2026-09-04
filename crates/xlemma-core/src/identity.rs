//! Privacy-preserving participant, operator, and node credential records.
//!
//! XLMP exposes pseudonymous identifiers, issuer attestations, qualification
//! claims, and revocation commitments. It deliberately contains no legal name,
//! passport, street-address, or other raw identity fields.

use crate::{
    canonical_json_hash, CredentialRevocationId, IdError, IssuerAttestationId, NodeCredentialId,
    NodeId, NodeRole, OperatorClusterId, OperatorCredentialId, OperatorId, PolicyId, ResearcherId,
    UserCredentialId, VerifiedUserId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialTier {
    V0Observer,
    V1VerifiedParticipant,
    V2VerifiedOperator,
    V3InstitutionalOperator,
    V4SpecializedAuthority,
}

impl CredentialTier {
    pub fn can_participate_in_consensus(self) -> bool {
        self >= Self::V2VerifiedOperator
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCredential {
    pub credential_id: UserCredentialId,
    pub verified_user_id: VerifiedUserId,
    pub researcher_id: Option<ResearcherId>,
    pub public_subject: String,
    pub tier: CredentialTier,
    pub issuer: String,
    pub uniqueness_commitment: String,
    pub qualifications: BTreeSet<String>,
    pub disclosure_policy: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub evidence_root: String,
    pub issuer_signature: String,
}

#[derive(Serialize)]
struct UserCredentialIdentity<'a> {
    verified_user_id: &'a VerifiedUserId,
    researcher_id: &'a Option<ResearcherId>,
    public_subject: &'a str,
    tier: CredentialTier,
    issuer: &'a str,
    uniqueness_commitment: &'a str,
    qualifications: &'a BTreeSet<String>,
    disclosure_policy: &'a str,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    evidence_root: &'a str,
}

impl UserCredential {
    pub fn derive_credential_id(&self) -> Result<UserCredentialId, IdError> {
        UserCredentialId::derive(&UserCredentialIdentity {
            verified_user_id: &self.verified_user_id,
            researcher_id: &self.researcher_id,
            public_subject: &self.public_subject,
            tier: self.tier,
            issuer: &self.issuer,
            uniqueness_commitment: &self.uniqueness_commitment,
            qualifications: &self.qualifications,
            disclosure_policy: &self.disclosure_policy,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            evidence_root: &self.evidence_root,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), CredentialError> {
        self.credential_id.validate()?;
        self.verified_user_id.validate()?;
        if let Some(researcher_id) = &self.researcher_id {
            researcher_id.validate()?;
        }
        if self.credential_id != self.derive_credential_id()? {
            return Err(CredentialError::IdentityMismatch);
        }
        if self.issued_at >= self.expires_at {
            return Err(CredentialError::ExpiredOrNotYetValid);
        }
        if self.public_subject.trim().is_empty()
            || self.issuer.trim().is_empty()
            || self.uniqueness_commitment.trim().is_empty()
            || self.disclosure_policy.trim().is_empty()
            || self.evidence_root.trim().is_empty()
            || self.issuer_signature.trim().is_empty()
        {
            return Err(CredentialError::MissingEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorCredential {
    pub credential_id: OperatorCredentialId,
    pub operator_id: OperatorId,
    pub verified_user_id: VerifiedUserId,
    pub user_credential_id: UserCredentialId,
    pub operator_cluster_id: OperatorClusterId,
    pub authorized_roles: BTreeSet<NodeRole>,
    pub qualifications: BTreeSet<String>,
    pub jurisdiction_class: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub evidence_root: String,
    pub holder_delegation_signature: String,
    pub issuer_signature: String,
}

#[derive(Serialize)]
struct OperatorCredentialIdentity<'a> {
    operator_id: &'a OperatorId,
    verified_user_id: &'a VerifiedUserId,
    user_credential_id: &'a UserCredentialId,
    operator_cluster_id: &'a OperatorClusterId,
    authorized_roles: &'a BTreeSet<NodeRole>,
    qualifications: &'a BTreeSet<String>,
    jurisdiction_class: &'a str,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    evidence_root: &'a str,
}

impl OperatorCredential {
    pub fn derive_credential_id(&self) -> Result<OperatorCredentialId, IdError> {
        OperatorCredentialId::derive(&OperatorCredentialIdentity {
            operator_id: &self.operator_id,
            verified_user_id: &self.verified_user_id,
            user_credential_id: &self.user_credential_id,
            operator_cluster_id: &self.operator_cluster_id,
            authorized_roles: &self.authorized_roles,
            qualifications: &self.qualifications,
            jurisdiction_class: &self.jurisdiction_class,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            evidence_root: &self.evidence_root,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), CredentialError> {
        self.credential_id.validate()?;
        self.operator_id.validate()?;
        self.verified_user_id.validate()?;
        self.user_credential_id.validate()?;
        self.operator_cluster_id.validate()?;
        if self.credential_id != self.derive_credential_id()? {
            return Err(CredentialError::IdentityMismatch);
        }
        if self.issued_at >= self.expires_at {
            return Err(CredentialError::ExpiredOrNotYetValid);
        }
        if self.authorized_roles.is_empty()
            || self.jurisdiction_class.trim().is_empty()
            || self.evidence_root.trim().is_empty()
            || self.holder_delegation_signature.trim().is_empty()
            || self.issuer_signature.trim().is_empty()
        {
            return Err(CredentialError::MissingEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCredential {
    pub credential_id: NodeCredentialId,
    pub node_id: NodeId,
    pub operator_id: OperatorId,
    pub operator_credential_id: OperatorCredentialId,
    pub operator_cluster_id: OperatorClusterId,
    pub node_public_key: String,
    pub authorized_roles: BTreeSet<NodeRole>,
    pub hardware_attestation_root: Option<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub evidence_root: String,
    pub operator_delegation_signature: String,
}

#[derive(Serialize)]
struct NodeCredentialIdentity<'a> {
    node_id: &'a NodeId,
    operator_id: &'a OperatorId,
    operator_credential_id: &'a OperatorCredentialId,
    operator_cluster_id: &'a OperatorClusterId,
    node_public_key: &'a str,
    authorized_roles: &'a BTreeSet<NodeRole>,
    hardware_attestation_root: &'a Option<String>,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    evidence_root: &'a str,
}

impl NodeCredential {
    pub fn derive_credential_id(&self) -> Result<NodeCredentialId, IdError> {
        NodeCredentialId::derive(&NodeCredentialIdentity {
            node_id: &self.node_id,
            operator_id: &self.operator_id,
            operator_credential_id: &self.operator_credential_id,
            operator_cluster_id: &self.operator_cluster_id,
            node_public_key: &self.node_public_key,
            authorized_roles: &self.authorized_roles,
            hardware_attestation_root: &self.hardware_attestation_root,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            evidence_root: &self.evidence_root,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), CredentialError> {
        self.credential_id.validate()?;
        self.node_id.validate()?;
        self.operator_id.validate()?;
        self.operator_credential_id.validate()?;
        self.operator_cluster_id.validate()?;
        if self.credential_id != self.derive_credential_id()? {
            return Err(CredentialError::IdentityMismatch);
        }
        if self.issued_at >= self.expires_at {
            return Err(CredentialError::ExpiredOrNotYetValid);
        }
        if self.authorized_roles.is_empty()
            || self.node_public_key.trim().is_empty()
            || self.evidence_root.trim().is_empty()
            || self.operator_delegation_signature.trim().is_empty()
        {
            return Err(CredentialError::MissingEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "credential_kind",
    content = "credential_id",
    rename_all = "snake_case"
)]
pub enum CredentialReference {
    User(UserCredentialId),
    Operator(OperatorCredentialId),
    Node(NodeCredentialId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRevocation {
    pub revocation_id: CredentialRevocationId,
    pub credential: CredentialReference,
    pub effective_at: DateTime<Utc>,
    pub reason_code: String,
    pub evidence_root: String,
    pub issuer: String,
    pub issuer_signature: String,
}

#[derive(Serialize)]
struct CredentialRevocationIdentity<'a> {
    credential: &'a CredentialReference,
    effective_at: DateTime<Utc>,
    reason_code: &'a str,
    evidence_root: &'a str,
    issuer: &'a str,
}

impl CredentialRevocation {
    pub fn derive_revocation_id(&self) -> Result<CredentialRevocationId, IdError> {
        CredentialRevocationId::derive(&CredentialRevocationIdentity {
            credential: &self.credential,
            effective_at: self.effective_at,
            reason_code: &self.reason_code,
            evidence_root: &self.evidence_root,
            issuer: &self.issuer,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), CredentialError> {
        self.revocation_id.validate()?;
        match &self.credential {
            CredentialReference::User(id) => id.validate()?,
            CredentialReference::Operator(id) => id.validate()?,
            CredentialReference::Node(id) => id.validate()?,
        }
        if self.revocation_id != self.derive_revocation_id()? {
            return Err(CredentialError::IdentityMismatch);
        }
        if self.reason_code.trim().is_empty()
            || self.evidence_root.trim().is_empty()
            || self.issuer.trim().is_empty()
            || self.issuer_signature.trim().is_empty()
        {
            return Err(CredentialError::MissingEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialStatusProof {
    pub user_credential_id: UserCredentialId,
    pub operator_credential_id: OperatorCredentialId,
    pub node_credential_id: NodeCredentialId,
    pub revocation_registry_root: String,
    pub checked_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub non_revocation_proof: String,
    pub issuer_signature: String,
}

impl CredentialStatusProof {
    pub fn validate_integrity(&self) -> Result<(), CredentialError> {
        self.user_credential_id.validate()?;
        self.operator_credential_id.validate()?;
        self.node_credential_id.validate()?;
        if self.checked_at >= self.valid_until {
            return Err(CredentialError::ExpiredOrNotYetValid);
        }
        if self.revocation_registry_root.trim().is_empty()
            || self.non_revocation_proof.trim().is_empty()
            || self.issuer_signature.trim().is_empty()
        {
            return Err(CredentialError::MissingEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCredentialChain {
    pub user: UserCredential,
    pub operator: OperatorCredential,
    pub node: NodeCredential,
    pub status: CredentialStatusProof,
}

impl NodeCredentialChain {
    pub fn derive_chain_root(&self) -> Result<String, crate::CanonicalizationError> {
        let digest = canonical_json_hash(
            "xlemma-node-credential-chain-v1",
            &(
                &self.user.credential_id,
                &self.operator.credential_id,
                &self.node.credential_id,
                &self.status,
            ),
        )?;
        Ok(format!(
            "blake3:{}",
            blake3::Hash::from_bytes(digest).to_hex()
        ))
    }

    pub fn validate_for(
        &self,
        node_id: &NodeId,
        operator_cluster_id: &OperatorClusterId,
        role: NodeRole,
        minimum_tier: CredentialTier,
        required_qualifications: &BTreeSet<String>,
        at: DateTime<Utc>,
    ) -> Result<(), CredentialError> {
        self.user.validate_integrity()?;
        self.operator.validate_integrity()?;
        self.node.validate_integrity()?;
        self.status.validate_integrity()?;
        if self.operator.verified_user_id != self.user.verified_user_id
            || self.operator.user_credential_id != self.user.credential_id
            || self.node.operator_id != self.operator.operator_id
            || self.node.operator_credential_id != self.operator.credential_id
            || self.node.operator_cluster_id != self.operator.operator_cluster_id
            || &self.node.node_id != node_id
            || &self.node.operator_cluster_id != operator_cluster_id
        {
            return Err(CredentialError::ChainMismatch);
        }
        if !self
            .node
            .authorized_roles
            .is_subset(&self.operator.authorized_roles)
        {
            return Err(CredentialError::UnauthorizedRoleOrQualification);
        }
        if self.status.user_credential_id != self.user.credential_id
            || self.status.operator_credential_id != self.operator.credential_id
            || self.status.node_credential_id != self.node.credential_id
        {
            return Err(CredentialError::StatusMismatch);
        }
        if self.user.issued_at > at
            || self.operator.issued_at > at
            || self.node.issued_at > at
            || self.user.expires_at <= at
            || self.operator.expires_at <= at
            || self.node.expires_at <= at
            || self.operator.issued_at < self.user.issued_at
            || self.operator.expires_at > self.user.expires_at
            || self.node.issued_at < self.operator.issued_at
            || self.node.expires_at > self.operator.expires_at
            || self.status.checked_at < self.node.issued_at
            || self.status.checked_at > at
            || self.status.valid_until <= at
            || self.status.valid_until > self.node.expires_at
            || self.status.valid_until > self.operator.expires_at
            || self.status.valid_until > self.user.expires_at
        {
            return Err(CredentialError::ExpiredOrNotYetValid);
        }
        if self.user.tier < minimum_tier
            || (minimum_tier.can_participate_in_consensus()
                && !self.user.tier.can_participate_in_consensus())
        {
            return Err(CredentialError::InsufficientTier);
        }
        let qualifications = self
            .user
            .qualifications
            .union(&self.operator.qualifications)
            .cloned()
            .collect::<BTreeSet<_>>();
        if !self.operator.authorized_roles.contains(&role)
            || !self.node.authorized_roles.contains(&role)
            || !required_qualifications.is_subset(&qualifications)
        {
            return Err(CredentialError::UnauthorizedRoleOrQualification);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialClaimKind {
    Uniqueness,
    Organization,
    OperatorEligibility,
    RoleQualification,
    ConflictClearance,
    NonRevocation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialClaimRequirement {
    /// At least one claim in this set must be supplied by the required number
    /// of independent issuers. This supports uniqueness OR organization rules.
    pub any_of: BTreeSet<CredentialClaimKind>,
    pub minimum_distinct_issuers: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialIssuerPolicy {
    pub policy_id: PolicyId,
    /// Each issuer maps to the claim kinds it is authorized to attest.
    pub approved_issuers: BTreeMap<String, BTreeSet<CredentialClaimKind>>,
    pub requirements: Vec<CredentialClaimRequirement>,
    pub minimum_distinct_issuers: u16,
    pub maximum_issuer_attestation_share_bps: u16,
    pub revocation_transparency_roots: BTreeSet<String>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub signatures: Vec<String>,
}

#[derive(Serialize)]
struct IssuerPolicyIdentity<'a> {
    approved_issuers: &'a BTreeMap<String, BTreeSet<CredentialClaimKind>>,
    requirements: &'a [CredentialClaimRequirement],
    minimum_distinct_issuers: u16,
    maximum_issuer_attestation_share_bps: u16,
    revocation_transparency_roots: &'a BTreeSet<String>,
    valid_from: DateTime<Utc>,
    valid_until: DateTime<Utc>,
}

impl CredentialIssuerPolicy {
    pub fn derive_policy_id(&self) -> Result<PolicyId, IdError> {
        PolicyId::derive(&IssuerPolicyIdentity {
            approved_issuers: &self.approved_issuers,
            requirements: &self.requirements,
            minimum_distinct_issuers: self.minimum_distinct_issuers,
            maximum_issuer_attestation_share_bps: self.maximum_issuer_attestation_share_bps,
            revocation_transparency_roots: &self.revocation_transparency_roots,
            valid_from: self.valid_from,
            valid_until: self.valid_until,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), CredentialError> {
        self.policy_id.validate()?;
        if self.policy_id != self.derive_policy_id()? {
            return Err(CredentialError::IdentityMismatch);
        }
        if self.approved_issuers.len() < 2
            || self.minimum_distinct_issuers < 2
            || usize::from(self.minimum_distinct_issuers) > self.approved_issuers.len()
            || self.maximum_issuer_attestation_share_bps == 0
            || self.maximum_issuer_attestation_share_bps >= 10_000
            || self.valid_from >= self.valid_until
            || self.requirements.is_empty()
            || self.requirements.iter().any(|requirement| {
                requirement.any_of.is_empty()
                    || requirement.minimum_distinct_issuers == 0
                    || requirement.minimum_distinct_issuers > self.minimum_distinct_issuers
            })
            || self
                .approved_issuers
                .iter()
                .any(|(issuer, claims)| issuer.trim().is_empty() || claims.is_empty())
            || self.revocation_transparency_roots.is_empty()
            || self
                .revocation_transparency_roots
                .iter()
                .any(|root| root.trim().is_empty())
            || self.signatures.is_empty()
            || self.signatures.iter().any(|value| value.trim().is_empty())
        {
            return Err(CredentialError::InvalidIssuerPolicy);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependentCredentialAttestation {
    pub attestation_id: IssuerAttestationId,
    pub subject: VerifiedUserId,
    pub issuer: String,
    pub claims: BTreeSet<CredentialClaimKind>,
    pub qualifications: BTreeSet<String>,
    pub selective_disclosure_proof: String,
    pub status_registry_root: String,
    pub non_revocation_proof: String,
    pub evidence_root: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub issuer_signature: String,
}

#[derive(Serialize)]
struct IssuerAttestationIdentity<'a> {
    subject: &'a VerifiedUserId,
    issuer: &'a str,
    claims: &'a BTreeSet<CredentialClaimKind>,
    qualifications: &'a BTreeSet<String>,
    selective_disclosure_proof: &'a str,
    status_registry_root: &'a str,
    non_revocation_proof: &'a str,
    evidence_root: &'a str,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl IndependentCredentialAttestation {
    pub fn derive_attestation_id(&self) -> Result<IssuerAttestationId, IdError> {
        IssuerAttestationId::derive(&IssuerAttestationIdentity {
            subject: &self.subject,
            issuer: &self.issuer,
            claims: &self.claims,
            qualifications: &self.qualifications,
            selective_disclosure_proof: &self.selective_disclosure_proof,
            status_registry_root: &self.status_registry_root,
            non_revocation_proof: &self.non_revocation_proof,
            evidence_root: &self.evidence_root,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), CredentialError> {
        self.attestation_id.validate()?;
        self.subject.validate()?;
        if self.attestation_id != self.derive_attestation_id()? {
            return Err(CredentialError::IdentityMismatch);
        }
        if self.issuer.trim().is_empty()
            || self.claims.is_empty()
            || self.selective_disclosure_proof.trim().is_empty()
            || self.status_registry_root.trim().is_empty()
            || self.non_revocation_proof.trim().is_empty()
            || self.evidence_root.trim().is_empty()
            || self.issuer_signature.trim().is_empty()
            || self.issued_at >= self.expires_at
            || (self
                .claims
                .contains(&CredentialClaimKind::RoleQualification)
                && self.qualifications.is_empty())
        {
            return Err(CredentialError::MissingEvidence);
        }
        Ok(())
    }
}

pub fn validate_independent_credential_set(
    policy: &CredentialIssuerPolicy,
    subject: &VerifiedUserId,
    attestations: &[IndependentCredentialAttestation],
    at: DateTime<Utc>,
) -> Result<(), CredentialError> {
    policy.validate_integrity()?;
    subject.validate()?;
    if at < policy.valid_from || at >= policy.valid_until || attestations.is_empty() {
        return Err(CredentialError::ExpiredOrNotYetValid);
    }
    let mut attestation_ids = BTreeSet::new();
    let mut issuer_counts = BTreeMap::<String, u32>::new();
    for attestation in attestations {
        attestation.validate_integrity()?;
        if &attestation.subject != subject
            || attestation.issued_at > at
            || attestation.expires_at <= at
            || !attestation_ids.insert(attestation.attestation_id.clone())
        {
            return Err(CredentialError::InvalidIssuerAttestation);
        }
        let authorized_claims = policy
            .approved_issuers
            .get(&attestation.issuer)
            .ok_or(CredentialError::UnapprovedIssuer)?;
        if !attestation.claims.is_subset(authorized_claims) {
            return Err(CredentialError::UnapprovedIssuer);
        }
        *issuer_counts.entry(attestation.issuer.clone()).or_default() += 1;
    }
    if issuer_counts.len() < usize::from(policy.minimum_distinct_issuers) {
        return Err(CredentialError::IssuerConcentration);
    }
    let total =
        u64::try_from(attestations.len()).map_err(|_| CredentialError::IssuerConcentration)?;
    for count in issuer_counts.values() {
        let count = u64::from(*count);
        let share_bps = count
            .checked_mul(10_000)
            .and_then(|scaled| scaled.checked_add(total - 1))
            .ok_or(CredentialError::IssuerConcentration)?
            / total;
        if share_bps > u64::from(policy.maximum_issuer_attestation_share_bps) {
            return Err(CredentialError::IssuerConcentration);
        }
    }
    for requirement in &policy.requirements {
        let issuers = attestations
            .iter()
            .filter(|attestation| !attestation.claims.is_disjoint(&requirement.any_of))
            .map(|attestation| attestation.issuer.as_str())
            .collect::<BTreeSet<_>>();
        if issuers.len() < usize::from(requirement.minimum_distinct_issuers) {
            return Err(CredentialError::MissingRequiredClaim);
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("credential content does not match its content-derived identifier")]
    IdentityMismatch,
    #[error("participant, operator, and node delegation links do not match")]
    ChainMismatch,
    #[error("non-revocation status proof does not bind the credential chain")]
    StatusMismatch,
    #[error("credential or status proof is expired or not yet valid")]
    ExpiredOrNotYetValid,
    #[error("credential tier is insufficient for the requested role")]
    InsufficientTier,
    #[error("credential does not authorize the role or required qualifications")]
    UnauthorizedRoleOrQualification,
    #[error("credential evidence, delegation, non-revocation proof, or signature is absent")]
    MissingEvidence,
    #[error("credential issuer policy is concentrated, incomplete, or invalid")]
    InvalidIssuerPolicy,
    #[error("credential attestation is duplicated, expired, or bound to another subject")]
    InvalidIssuerAttestation,
    #[error("credential attestation was issued outside its authorized claim scope")]
    UnapprovedIssuer,
    #[error("credential evidence depends on too few independent issuers")]
    IssuerConcentration,
    #[error("credential set does not satisfy a required independent claim group")]
    MissingRequiredClaim,
    #[error(transparent)]
    Identifier(#[from] IdError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn credential_chain(tier: CredentialTier, now: DateTime<Utc>) -> NodeCredentialChain {
        let verified_user_id = VerifiedUserId::derive(&"pseudonymous-participant").unwrap();
        let operator_id = OperatorId::derive(&"operator").unwrap();
        let operator_cluster_id = OperatorClusterId::derive(&"operator-cluster").unwrap();
        let node_id = NodeId::derive(&"node").unwrap();
        let mut user = UserCredential {
            credential_id: UserCredentialId::derive(&"pending").unwrap(),
            verified_user_id: verified_user_id.clone(),
            researcher_id: None,
            public_subject: "did:key:pseudonymous-participant".into(),
            tier,
            issuer: "did:web:credential-issuer.example".into(),
            uniqueness_commitment: "blake3:private-uniqueness-proof".into(),
            qualifications: BTreeSet::from(["formal-verification".into()]),
            disclosure_policy: "pseudonymous-v1".into(),
            issued_at: now - Duration::days(1),
            expires_at: now + Duration::days(30),
            evidence_root: "blake3:user-evidence-root".into(),
            issuer_signature: "issuer-signature".into(),
        };
        user.credential_id = user.derive_credential_id().unwrap();
        let mut operator = OperatorCredential {
            credential_id: OperatorCredentialId::derive(&"pending").unwrap(),
            operator_id: operator_id.clone(),
            verified_user_id,
            user_credential_id: user.credential_id.clone(),
            operator_cluster_id: operator_cluster_id.clone(),
            authorized_roles: BTreeSet::from([NodeRole::OfficialKernelChecker]),
            qualifications: BTreeSet::from(["lean-kernel".into()]),
            jurisdiction_class: "privacy-preserving-verified".into(),
            issued_at: user.issued_at,
            expires_at: user.expires_at,
            evidence_root: "blake3:operator-evidence-root".into(),
            holder_delegation_signature: "holder-delegation-signature".into(),
            issuer_signature: "issuer-signature".into(),
        };
        operator.credential_id = operator.derive_credential_id().unwrap();
        let mut node = NodeCredential {
            credential_id: NodeCredentialId::derive(&"pending").unwrap(),
            node_id,
            operator_id,
            operator_credential_id: operator.credential_id.clone(),
            operator_cluster_id,
            node_public_key: "did:key:node".into(),
            authorized_roles: BTreeSet::from([NodeRole::OfficialKernelChecker]),
            hardware_attestation_root: None,
            issued_at: operator.issued_at,
            expires_at: operator.expires_at,
            evidence_root: "blake3:node-evidence-root".into(),
            operator_delegation_signature: "operator-delegation-signature".into(),
        };
        node.credential_id = node.derive_credential_id().unwrap();
        let status = CredentialStatusProof {
            user_credential_id: user.credential_id.clone(),
            operator_credential_id: operator.credential_id.clone(),
            node_credential_id: node.credential_id.clone(),
            revocation_registry_root: "blake3:revocation-registry-root".into(),
            checked_at: now,
            valid_until: now + Duration::hours(1),
            non_revocation_proof: "non-revocation-proof".into(),
            issuer_signature: "status-issuer-signature".into(),
        };
        NodeCredentialChain {
            user,
            operator,
            node,
            status,
        }
    }

    #[test]
    fn pseudonymous_v2_chain_authorizes_consensus_without_public_legal_identity() {
        let now = Utc::now();
        let chain = credential_chain(CredentialTier::V2VerifiedOperator, now);
        chain
            .validate_for(
                &chain.node.node_id,
                &chain.node.operator_cluster_id,
                NodeRole::OfficialKernelChecker,
                CredentialTier::V2VerifiedOperator,
                &BTreeSet::from(["lean-kernel".into()]),
                now,
            )
            .unwrap();
        assert!(chain.user.researcher_id.is_none());
        let public_json = serde_json::to_string(&chain.user).unwrap();
        for forbidden in ["legal_name", "passport", "street_address", "biometric"] {
            assert!(!public_json.contains(forbidden));
        }
    }

    #[test]
    fn v1_participant_cannot_enter_poir_consensus() {
        let now = Utc::now();
        let chain = credential_chain(CredentialTier::V1VerifiedParticipant, now);
        assert!(matches!(
            chain.validate_for(
                &chain.node.node_id,
                &chain.node.operator_cluster_id,
                NodeRole::OfficialKernelChecker,
                CredentialTier::V2VerifiedOperator,
                &BTreeSet::new(),
                now,
            ),
            Err(CredentialError::InsufficientTier)
        ));
    }

    #[test]
    fn gold_eligibility_can_require_multiple_selective_disclosure_issuers() {
        let now = Utc::now();
        let subject = VerifiedUserId::derive(&"pseudonymous-gold-operator").unwrap();
        let issuer_claims = [
            (
                "did:web:uniqueness.example",
                BTreeSet::from([
                    CredentialClaimKind::Uniqueness,
                    CredentialClaimKind::NonRevocation,
                ]),
            ),
            (
                "did:web:operator.example",
                BTreeSet::from([
                    CredentialClaimKind::OperatorEligibility,
                    CredentialClaimKind::ConflictClearance,
                ]),
            ),
            (
                "did:web:qualification.example",
                BTreeSet::from([CredentialClaimKind::RoleQualification]),
            ),
        ];
        let mut policy = CredentialIssuerPolicy {
            policy_id: PolicyId::derive(&"placeholder").unwrap(),
            approved_issuers: issuer_claims
                .iter()
                .map(|(issuer, claims)| ((*issuer).into(), claims.clone()))
                .collect(),
            requirements: vec![
                CredentialClaimRequirement {
                    any_of: BTreeSet::from([
                        CredentialClaimKind::Uniqueness,
                        CredentialClaimKind::Organization,
                    ]),
                    minimum_distinct_issuers: 1,
                },
                CredentialClaimRequirement {
                    any_of: BTreeSet::from([CredentialClaimKind::OperatorEligibility]),
                    minimum_distinct_issuers: 1,
                },
                CredentialClaimRequirement {
                    any_of: BTreeSet::from([CredentialClaimKind::RoleQualification]),
                    minimum_distinct_issuers: 1,
                },
                CredentialClaimRequirement {
                    any_of: BTreeSet::from([CredentialClaimKind::ConflictClearance]),
                    minimum_distinct_issuers: 1,
                },
            ],
            minimum_distinct_issuers: 3,
            maximum_issuer_attestation_share_bps: 3_334,
            revocation_transparency_roots: BTreeSet::from(["blake3:public-revocation-root".into()]),
            valid_from: now - Duration::days(1),
            valid_until: now + Duration::days(30),
            signatures: vec!["policy-signature".into()],
        };
        policy.policy_id = policy.derive_policy_id().unwrap();

        let attestations = issuer_claims
            .into_iter()
            .map(|(issuer, claims)| {
                let mut attestation = IndependentCredentialAttestation {
                    attestation_id: IssuerAttestationId::derive(&"placeholder").unwrap(),
                    subject: subject.clone(),
                    issuer: issuer.into(),
                    qualifications: claims
                        .contains(&CredentialClaimKind::RoleQualification)
                        .then(|| BTreeSet::from(["lean-kernel".into()]))
                        .unwrap_or_default(),
                    claims,
                    selective_disclosure_proof: "zk:selective-disclosure".into(),
                    status_registry_root: "blake3:issuer-status".into(),
                    non_revocation_proof: "accumulator:non-revocation".into(),
                    evidence_root: "blake3:issuer-private-evidence".into(),
                    issued_at: now - Duration::hours(1),
                    expires_at: now + Duration::days(7),
                    issuer_signature: "issuer-signature".into(),
                };
                attestation.attestation_id = attestation.derive_attestation_id().unwrap();
                attestation
            })
            .collect::<Vec<_>>();

        validate_independent_credential_set(&policy, &subject, &attestations, now).unwrap();
        assert!(matches!(
            validate_independent_credential_set(&policy, &subject, &attestations[..1], now),
            Err(CredentialError::IssuerConcentration)
        ));
        let public = serde_json::to_string(&attestations).unwrap();
        for forbidden in ["legal_name", "passport", "street_address", "biometric"] {
            assert!(!public.contains(forbidden));
        }
    }
}
