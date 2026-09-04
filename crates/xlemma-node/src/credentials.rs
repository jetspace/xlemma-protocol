//! Append-only credential and revocation registry for node admission.

use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use thiserror::Error;
use xlemma_consensus::CommitteeAdmissionVerifier;
use xlemma_core::{
    canonical_json_hash, CommitteeRequirement, CredentialReference, CredentialRevocation,
    CredentialRevocationId, CredentialTier, EligibleNode, NodeCredential, NodeCredentialChain,
    NodeCredentialId, NodeRole, OperatorCredential, OperatorCredentialId, UserCredential,
    UserCredentialId,
};

/// Cryptographic verification is deliberately adapter-owned. Implementations
/// must verify issuer and delegation signatures against the exact
/// content-derived credential identifiers and verify the status proof against
/// its committed revocation-registry root.
pub trait CredentialProofVerifier {
    fn verify_user(&self, credential: &UserCredential) -> bool;
    fn verify_operator(&self, credential: &OperatorCredential) -> bool;
    fn verify_node(&self, credential: &NodeCredential) -> bool;
    fn verify_revocation(&self, revocation: &CredentialRevocation) -> bool;
    fn verify_status(&self, chain: &NodeCredentialChain) -> bool;
}

#[derive(Debug, Error)]
pub enum CredentialRegistryError {
    #[error("credential or revocation identifier already exists")]
    DuplicateIdentifier,
    #[error("credential or revocation failed structural validation")]
    InvalidRecord,
    #[error("credential issuer, delegation, revocation, or status proof signature is invalid")]
    InvalidProof,
    #[error("credential delegation parent is absent or does not match")]
    MissingOrMismatchedParent,
    #[error("credential chain is not the exact append-only registry record")]
    UnregisteredChain,
    #[error("credential is revoked at the validation time")]
    Revoked,
    #[error("status proof does not commit to the current revocation registry")]
    StaleRevocationRoot,
    #[error(transparent)]
    Canonicalization(#[from] xlemma_core::CanonicalizationError),
}

#[derive(Default)]
pub struct CredentialRegistry {
    users: BTreeMap<UserCredentialId, UserCredential>,
    operators: BTreeMap<OperatorCredentialId, OperatorCredential>,
    nodes: BTreeMap<NodeCredentialId, NodeCredential>,
    revocations: BTreeMap<CredentialRevocationId, CredentialRevocation>,
}

/// Bridges append-only credential registration and cryptographic proof checks
/// into committee sortition. A candidate is never admitted from structural
/// credential strings alone.
pub struct RegistryCommitteeAdmission<'a, V> {
    registry: &'a CredentialRegistry,
    proof_verifier: &'a V,
}

impl<'a, V> RegistryCommitteeAdmission<'a, V> {
    pub fn new(registry: &'a CredentialRegistry, proof_verifier: &'a V) -> Self {
        Self {
            registry,
            proof_verifier,
        }
    }
}

impl<V: CredentialProofVerifier> CommitteeAdmissionVerifier for RegistryCommitteeAdmission<'_, V> {
    fn verify_candidate(
        &self,
        node: &EligibleNode,
        requirement: &CommitteeRequirement,
        selected_at: DateTime<Utc>,
    ) -> bool {
        self.registry
            .validate_chain(
                &node.credential_chain,
                requirement.role,
                requirement.minimum_credential_tier,
                &requirement.required_qualifications,
                selected_at,
                self.proof_verifier,
            )
            .is_ok()
    }
}

impl CredentialRegistry {
    pub fn publish_user(
        &mut self,
        credential: UserCredential,
        verifier: &impl CredentialProofVerifier,
    ) -> Result<(), CredentialRegistryError> {
        if credential.validate_integrity().is_err() {
            return Err(CredentialRegistryError::InvalidRecord);
        }
        if !verifier.verify_user(&credential) {
            return Err(CredentialRegistryError::InvalidProof);
        }
        if self.users.contains_key(&credential.credential_id) {
            return Err(CredentialRegistryError::DuplicateIdentifier);
        }
        self.users
            .insert(credential.credential_id.clone(), credential);
        Ok(())
    }

    pub fn publish_operator(
        &mut self,
        credential: OperatorCredential,
        verifier: &impl CredentialProofVerifier,
    ) -> Result<(), CredentialRegistryError> {
        if credential.validate_integrity().is_err() {
            return Err(CredentialRegistryError::InvalidRecord);
        }
        let Some(user) = self.users.get(&credential.user_credential_id) else {
            return Err(CredentialRegistryError::MissingOrMismatchedParent);
        };
        if user.verified_user_id != credential.verified_user_id
            || credential.issued_at < user.issued_at
            || credential.expires_at > user.expires_at
            || self.operators.values().any(|existing| {
                existing.operator_id == credential.operator_id
                    && (existing.verified_user_id != credential.verified_user_id
                        || existing.operator_cluster_id != credential.operator_cluster_id)
            })
        {
            return Err(CredentialRegistryError::MissingOrMismatchedParent);
        }
        if !verifier.verify_operator(&credential) {
            return Err(CredentialRegistryError::InvalidProof);
        }
        if self.operators.contains_key(&credential.credential_id) {
            return Err(CredentialRegistryError::DuplicateIdentifier);
        }
        self.operators
            .insert(credential.credential_id.clone(), credential);
        Ok(())
    }

    pub fn publish_node(
        &mut self,
        credential: NodeCredential,
        verifier: &impl CredentialProofVerifier,
    ) -> Result<(), CredentialRegistryError> {
        if credential.validate_integrity().is_err() {
            return Err(CredentialRegistryError::InvalidRecord);
        }
        let Some(operator) = self.operators.get(&credential.operator_credential_id) else {
            return Err(CredentialRegistryError::MissingOrMismatchedParent);
        };
        if operator.operator_id != credential.operator_id
            || operator.operator_cluster_id != credential.operator_cluster_id
            || credential.issued_at < operator.issued_at
            || credential.expires_at > operator.expires_at
        {
            return Err(CredentialRegistryError::MissingOrMismatchedParent);
        }
        if self.nodes.values().any(|existing| {
            existing.node_id == credential.node_id && existing.operator_id != credential.operator_id
        }) {
            return Err(CredentialRegistryError::MissingOrMismatchedParent);
        }
        if !verifier.verify_node(&credential) {
            return Err(CredentialRegistryError::InvalidProof);
        }
        if self.nodes.contains_key(&credential.credential_id) {
            return Err(CredentialRegistryError::DuplicateIdentifier);
        }
        self.nodes
            .insert(credential.credential_id.clone(), credential);
        Ok(())
    }

    pub fn publish_revocation(
        &mut self,
        revocation: CredentialRevocation,
        verifier: &impl CredentialProofVerifier,
    ) -> Result<(), CredentialRegistryError> {
        if revocation.validate_integrity().is_err() || !self.contains(&revocation.credential) {
            return Err(CredentialRegistryError::InvalidRecord);
        }
        if !verifier.verify_revocation(&revocation) {
            return Err(CredentialRegistryError::InvalidProof);
        }
        if self.revocations.contains_key(&revocation.revocation_id) {
            return Err(CredentialRegistryError::DuplicateIdentifier);
        }
        self.revocations
            .insert(revocation.revocation_id.clone(), revocation);
        Ok(())
    }

    pub fn revocation_registry_root(&self) -> Result<String, CredentialRegistryError> {
        let records = self.revocations.values().collect::<Vec<_>>();
        let digest = canonical_json_hash("xlemma-credential-revocation-registry-v1", &records)?;
        Ok(format!(
            "blake3:{}",
            blake3::Hash::from_bytes(digest).to_hex()
        ))
    }

    pub fn validate_chain(
        &self,
        chain: &NodeCredentialChain,
        role: NodeRole,
        minimum_tier: CredentialTier,
        required_qualifications: &std::collections::BTreeSet<String>,
        at: DateTime<Utc>,
        verifier: &impl CredentialProofVerifier,
    ) -> Result<(), CredentialRegistryError> {
        chain
            .validate_for(
                &chain.node.node_id,
                &chain.node.operator_cluster_id,
                role,
                minimum_tier,
                required_qualifications,
                at,
            )
            .map_err(|_| CredentialRegistryError::InvalidRecord)?;
        if self.users.get(&chain.user.credential_id) != Some(&chain.user)
            || self.operators.get(&chain.operator.credential_id) != Some(&chain.operator)
            || self.nodes.get(&chain.node.credential_id) != Some(&chain.node)
        {
            return Err(CredentialRegistryError::UnregisteredChain);
        }
        if self.chain_is_revoked(chain, at) {
            return Err(CredentialRegistryError::Revoked);
        }
        if chain.status.revocation_registry_root != self.revocation_registry_root()? {
            return Err(CredentialRegistryError::StaleRevocationRoot);
        }
        if !verifier.verify_status(chain) {
            return Err(CredentialRegistryError::InvalidProof);
        }
        Ok(())
    }

    fn contains(&self, reference: &CredentialReference) -> bool {
        match reference {
            CredentialReference::User(id) => self.users.contains_key(id),
            CredentialReference::Operator(id) => self.operators.contains_key(id),
            CredentialReference::Node(id) => self.nodes.contains_key(id),
        }
    }

    fn chain_is_revoked(&self, chain: &NodeCredentialChain, at: DateTime<Utc>) -> bool {
        self.revocations.values().any(|revocation| {
            revocation.effective_at <= at
                && match &revocation.credential {
                    CredentialReference::User(id) => id == &chain.user.credential_id,
                    CredentialReference::Operator(id) => id == &chain.operator.credential_id,
                    CredentialReference::Node(id) => id == &chain.node.credential_id,
                }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    struct AcceptExampleProofs;
    struct RejectProofs;

    impl CredentialProofVerifier for AcceptExampleProofs {
        fn verify_user(&self, _: &UserCredential) -> bool {
            true
        }

        fn verify_operator(&self, _: &OperatorCredential) -> bool {
            true
        }

        fn verify_node(&self, _: &NodeCredential) -> bool {
            true
        }

        fn verify_revocation(&self, _: &CredentialRevocation) -> bool {
            true
        }

        fn verify_status(&self, _: &NodeCredentialChain) -> bool {
            true
        }
    }

    impl CredentialProofVerifier for RejectProofs {
        fn verify_user(&self, _: &UserCredential) -> bool {
            false
        }

        fn verify_operator(&self, _: &OperatorCredential) -> bool {
            false
        }

        fn verify_node(&self, _: &NodeCredential) -> bool {
            false
        }

        fn verify_revocation(&self, _: &CredentialRevocation) -> bool {
            false
        }

        fn verify_status(&self, _: &NodeCredentialChain) -> bool {
            false
        }
    }

    fn example_chain() -> NodeCredentialChain {
        serde_json::from_str(include_str!(
            "../../../examples/node-network/credential-chain.json"
        ))
        .unwrap()
    }

    #[test]
    fn registry_admits_exact_chain_then_revocation_fails_closed() {
        let verifier = AcceptExampleProofs;
        let chain = example_chain();
        let at = "2026-09-03T12:01:00Z".parse().unwrap();
        let mut registry = CredentialRegistry::default();
        registry
            .publish_user(chain.user.clone(), &verifier)
            .unwrap();
        registry
            .publish_operator(chain.operator.clone(), &verifier)
            .unwrap();
        registry
            .publish_node(chain.node.clone(), &verifier)
            .unwrap();
        assert_eq!(
            registry.revocation_registry_root().unwrap(),
            chain.status.revocation_registry_root
        );
        registry
            .validate_chain(
                &chain,
                NodeRole::OfficialKernelChecker,
                CredentialTier::V2VerifiedOperator,
                &BTreeSet::from(["lean-kernel".into()]),
                at,
                &verifier,
            )
            .unwrap();

        let mut revocation = CredentialRevocation {
            revocation_id: CredentialRevocationId::derive(&"pending").unwrap(),
            credential: CredentialReference::Node(chain.node.credential_id.clone()),
            effective_at: "2026-09-03T12:00:30Z".parse().unwrap(),
            reason_code: "operator_requested_key_rotation".into(),
            evidence_root: "blake3:revocation-evidence-root".into(),
            issuer: "did:web:issuer.xlemma.example".into(),
            issuer_signature: "example-revocation-signature".into(),
        };
        revocation.revocation_id = revocation.derive_revocation_id().unwrap();
        registry.publish_revocation(revocation, &verifier).unwrap();
        assert!(matches!(
            registry.validate_chain(
                &chain,
                NodeRole::OfficialKernelChecker,
                CredentialTier::V2VerifiedOperator,
                &BTreeSet::new(),
                at,
                &verifier,
            ),
            Err(CredentialRegistryError::Revoked)
        ));
    }

    #[test]
    fn registry_is_append_only() {
        let verifier = AcceptExampleProofs;
        let chain = example_chain();
        let mut registry = CredentialRegistry::default();
        registry
            .publish_user(chain.user.clone(), &verifier)
            .unwrap();
        assert!(matches!(
            registry.publish_user(chain.user, &verifier),
            Err(CredentialRegistryError::DuplicateIdentifier)
        ));
    }

    #[test]
    fn registry_never_treats_nonempty_signature_text_as_verified() {
        let chain = example_chain();
        let mut registry = CredentialRegistry::default();
        assert!(matches!(
            registry.publish_user(chain.user, &RejectProofs),
            Err(CredentialRegistryError::InvalidProof)
        ));
    }
}
