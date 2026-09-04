//! Content-derived trust policies and axiom profiles.
//!
//! A registry says which formal environments a deployment is willing to
//! accept. It never makes a theorem true: it only determines whether exact
//! checker evidence satisfies an explicitly selected trust policy.

use crate::{
    canonical_json_hash, CheckerFamily, IdError, PolicyId, ProofManifest, TheoryManifest,
    XLMP_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomProfile {
    pub profile_id: PolicyId,
    pub protocol_version: String,
    pub name: String,
    pub permitted_axioms: BTreeSet<String>,
    pub explicitly_forbidden_axioms: BTreeSet<String>,
    pub allow_unlisted_axioms: bool,
    pub allow_sorry: bool,
    pub allow_unsafe_declarations: bool,
    pub allow_native_decide: bool,
    pub supersedes: Option<PolicyId>,
}

#[derive(Serialize)]
struct AxiomProfileIdentity<'a> {
    protocol_version: &'a str,
    name: &'a str,
    permitted_axioms: &'a BTreeSet<String>,
    explicitly_forbidden_axioms: &'a BTreeSet<String>,
    allow_unlisted_axioms: bool,
    allow_sorry: bool,
    allow_unsafe_declarations: bool,
    allow_native_decide: bool,
    supersedes: &'a Option<PolicyId>,
}

impl AxiomProfile {
    pub fn derive_profile_id(&self) -> Result<PolicyId, IdError> {
        PolicyId::derive(&AxiomProfileIdentity {
            protocol_version: &self.protocol_version,
            name: &self.name,
            permitted_axioms: &self.permitted_axioms,
            explicitly_forbidden_axioms: &self.explicitly_forbidden_axioms,
            allow_unlisted_axioms: self.allow_unlisted_axioms,
            allow_sorry: self.allow_sorry,
            allow_unsafe_declarations: self.allow_unsafe_declarations,
            allow_native_decide: self.allow_native_decide,
            supersedes: &self.supersedes,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), TrustPolicyError> {
        self.profile_id.validate()?;
        if self.protocol_version != XLMP_VERSION || self.name.trim().is_empty() {
            return Err(TrustPolicyError::InvalidProfile);
        }
        if self
            .permitted_axioms
            .iter()
            .chain(self.explicitly_forbidden_axioms.iter())
            .any(|axiom| axiom.trim().is_empty())
            || !self
                .permitted_axioms
                .is_disjoint(&self.explicitly_forbidden_axioms)
        {
            return Err(TrustPolicyError::InvalidProfile);
        }
        if self.allow_unlisted_axioms
            || self.allow_sorry
            || self.allow_unsafe_declarations
            || self.allow_native_decide
        {
            return Err(TrustPolicyError::UnsafeProfile);
        }
        if let Some(parent) = &self.supersedes {
            parent.validate()?;
            if parent == &self.profile_id {
                return Err(TrustPolicyError::InvalidProfile);
            }
        }
        if self.profile_id != self.derive_profile_id()? {
            return Err(TrustPolicyError::ProfileIdMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustPolicy {
    pub policy_id: PolicyId,
    pub protocol_version: String,
    pub name: String,
    pub axiom_profile_id: PolicyId,
    pub permitted_checker_families: BTreeSet<CheckerFamily>,
    pub minimum_independent_checker_families: usize,
    pub permitted_canonical_encodings: BTreeSet<String>,
    pub require_exact_challenge: bool,
    pub require_pinned_toolchain: bool,
    pub require_dependency_lock: bool,
    pub supersedes: Option<PolicyId>,
}

#[derive(Serialize)]
struct TrustPolicyIdentity<'a> {
    protocol_version: &'a str,
    name: &'a str,
    axiom_profile_id: &'a PolicyId,
    permitted_checker_families: &'a BTreeSet<CheckerFamily>,
    minimum_independent_checker_families: usize,
    permitted_canonical_encodings: &'a BTreeSet<String>,
    require_exact_challenge: bool,
    require_pinned_toolchain: bool,
    require_dependency_lock: bool,
    supersedes: &'a Option<PolicyId>,
}

impl TrustPolicy {
    pub fn derive_policy_id(&self) -> Result<PolicyId, IdError> {
        PolicyId::derive(&TrustPolicyIdentity {
            protocol_version: &self.protocol_version,
            name: &self.name,
            axiom_profile_id: &self.axiom_profile_id,
            permitted_checker_families: &self.permitted_checker_families,
            minimum_independent_checker_families: self.minimum_independent_checker_families,
            permitted_canonical_encodings: &self.permitted_canonical_encodings,
            require_exact_challenge: self.require_exact_challenge,
            require_pinned_toolchain: self.require_pinned_toolchain,
            require_dependency_lock: self.require_dependency_lock,
            supersedes: &self.supersedes,
        })
    }

    pub fn validate_integrity(&self) -> Result<(), TrustPolicyError> {
        self.policy_id.validate()?;
        self.axiom_profile_id.validate()?;
        if self.protocol_version != XLMP_VERSION
            || self.name.trim().is_empty()
            || self.permitted_checker_families.is_empty()
            || self.minimum_independent_checker_families == 0
            || self.minimum_independent_checker_families > self.permitted_checker_families.len()
            || self.permitted_canonical_encodings.is_empty()
            || self
                .permitted_canonical_encodings
                .iter()
                .any(|encoding| encoding.trim().is_empty())
        {
            return Err(TrustPolicyError::InvalidPolicy);
        }
        // Formal policies accepted by the reference registry are fail-closed.
        // Experimental workflows may define records elsewhere, but cannot be
        // mistaken for a certification-eligible trust policy.
        if !self.require_exact_challenge
            || !self.require_pinned_toolchain
            || !self.require_dependency_lock
        {
            return Err(TrustPolicyError::UnsafePolicy);
        }
        if let Some(parent) = &self.supersedes {
            parent.validate()?;
            if parent == &self.policy_id {
                return Err(TrustPolicyError::InvalidPolicy);
            }
        }
        if self.policy_id != self.derive_policy_id()? {
            return Err(TrustPolicyError::PolicyIdMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustPolicyRegistrySnapshot {
    pub protocol_version: String,
    pub registry_root: String,
    pub axiom_profiles: Vec<AxiomProfile>,
    pub trust_policies: Vec<TrustPolicy>,
}

#[derive(Serialize)]
struct RegistryIdentity<'a> {
    protocol_version: &'a str,
    axiom_profiles: &'a [AxiomProfile],
    trust_policies: &'a [TrustPolicy],
}

impl TrustPolicyRegistrySnapshot {
    pub fn expected_registry_root(&self) -> Result<String, crate::CanonicalizationError> {
        let digest = canonical_json_hash(
            "trust-policy-registry-v1",
            &RegistryIdentity {
                protocol_version: &self.protocol_version,
                axiom_profiles: &self.axiom_profiles,
                trust_policies: &self.trust_policies,
            },
        )?;
        Ok(format!("blake3:{}", hex::encode(digest)))
    }
}

#[derive(Clone, Debug, Default)]
pub struct TrustPolicyRegistry {
    profiles: BTreeMap<PolicyId, AxiomProfile>,
    policies: BTreeMap<PolicyId, TrustPolicy>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofTrustEvidence {
    pub observed_axioms: BTreeSet<String>,
    pub used_sorry: bool,
    pub used_unsafe_declarations: bool,
    pub used_native_decide: bool,
    pub checker_families: BTreeSet<CheckerFamily>,
    pub exact_challenge_matched: bool,
    pub toolchain_pinned: bool,
    pub dependency_lock_verified: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustEvaluation {
    pub policy_id: PolicyId,
    pub axiom_profile_id: PolicyId,
    pub accepted: bool,
    pub reasons: Vec<String>,
}

impl TrustPolicyRegistry {
    pub fn from_snapshot(snapshot: TrustPolicyRegistrySnapshot) -> Result<Self, TrustPolicyError> {
        if snapshot.protocol_version != XLMP_VERSION
            || snapshot.registry_root != snapshot.expected_registry_root()?
        {
            return Err(TrustPolicyError::RegistryRootMismatch);
        }
        if snapshot.axiom_profiles.is_empty() || snapshot.trust_policies.is_empty() {
            return Err(TrustPolicyError::EmptyRegistry);
        }
        if !snapshot
            .axiom_profiles
            .windows(2)
            .all(|pair| pair[0].profile_id < pair[1].profile_id)
            || !snapshot
                .trust_policies
                .windows(2)
                .all(|pair| pair[0].policy_id < pair[1].policy_id)
        {
            return Err(TrustPolicyError::RegistryNotCanonical);
        }

        let mut registry = Self::default();
        for profile in snapshot.axiom_profiles {
            registry.register_profile(profile)?;
        }
        for policy in snapshot.trust_policies {
            registry.register_policy(policy)?;
        }
        Ok(registry)
    }

    pub fn register_profile(&mut self, profile: AxiomProfile) -> Result<(), TrustPolicyError> {
        profile.validate_integrity()?;
        if self.profiles.contains_key(&profile.profile_id) {
            return Err(TrustPolicyError::DuplicateObject);
        }
        self.profiles.insert(profile.profile_id.clone(), profile);
        Ok(())
    }

    pub fn register_policy(&mut self, policy: TrustPolicy) -> Result<(), TrustPolicyError> {
        policy.validate_integrity()?;
        if !self.profiles.contains_key(&policy.axiom_profile_id) {
            return Err(TrustPolicyError::UnknownAxiomProfile);
        }
        if self.policies.contains_key(&policy.policy_id) {
            return Err(TrustPolicyError::DuplicateObject);
        }
        self.policies.insert(policy.policy_id.clone(), policy);
        Ok(())
    }

    pub fn evaluate(
        &self,
        theory: &TheoryManifest,
        proof: &ProofManifest,
        evidence: &ProofTrustEvidence,
    ) -> Result<TrustEvaluation, TrustPolicyError> {
        let policy = self
            .policies
            .get(&theory.trust_policy_id)
            .ok_or(TrustPolicyError::UnknownTrustPolicy)?;
        let profile = self
            .profiles
            .get(&policy.axiom_profile_id)
            .ok_or(TrustPolicyError::UnknownAxiomProfile)?;
        let mut reasons = Vec::new();

        if theory.protocol_version != XLMP_VERSION || proof.protocol_version != XLMP_VERSION {
            reasons.push("protocol version does not match XLMP/1".to_owned());
        }
        if !policy
            .permitted_canonical_encodings
            .contains(&theory.canonical_encoding)
        {
            reasons.push("theory canonical encoding is not permitted".to_owned());
        }
        if policy.require_exact_challenge && !evidence.exact_challenge_matched {
            reasons.push("exact challenge was not matched".to_owned());
        }
        if policy.require_pinned_toolchain
            && (!evidence.toolchain_pinned || theory.lean_toolchain.trim().is_empty())
        {
            reasons.push("toolchain is not pinned".to_owned());
        }
        if policy.require_dependency_lock
            && (!evidence.dependency_lock_verified
                || theory.dependency_merkle_root.trim().is_empty()
                || proof.dependency_root.trim().is_empty())
        {
            reasons.push("dependency lock was not verified".to_owned());
        }
        if evidence.checker_families.len() < policy.minimum_independent_checker_families
            || !evidence
                .checker_families
                .is_subset(&policy.permitted_checker_families)
        {
            reasons.push("checker-family evidence does not satisfy policy".to_owned());
        }
        if evidence.used_sorry && !profile.allow_sorry {
            reasons.push("proof uses sorry/admit".to_owned());
        }
        if evidence.used_unsafe_declarations && !profile.allow_unsafe_declarations {
            reasons.push("proof uses unsafe declarations".to_owned());
        }
        if evidence.used_native_decide && !profile.allow_native_decide {
            reasons.push("proof uses native_decide or compiler-trusted evaluation".to_owned());
        }

        let declared_axioms: BTreeSet<_> = theory.permitted_axioms.iter().cloned().collect();
        let proof_axioms: BTreeSet<_> = proof.observed_axioms.iter().cloned().collect();
        if declared_axioms.len() != theory.permitted_axioms.len()
            || proof_axioms.len() != proof.observed_axioms.len()
        {
            reasons.push("axiom inventories contain duplicate entries".to_owned());
        }
        for axiom in &declared_axioms {
            if profile.explicitly_forbidden_axioms.contains(axiom)
                || !profile.permitted_axioms.contains(axiom)
            {
                reasons.push(format!("theory declares an unapproved axiom: {axiom}"));
            }
        }
        if proof_axioms != evidence.observed_axioms {
            reasons.push("proof manifest and checker axiom inventories differ".to_owned());
        }
        for axiom in &evidence.observed_axioms {
            if profile.explicitly_forbidden_axioms.contains(axiom) {
                reasons.push(format!("axiom is explicitly forbidden: {axiom}"));
            } else if !profile.allow_unlisted_axioms
                && (!profile.permitted_axioms.contains(axiom) || !declared_axioms.contains(axiom))
            {
                reasons.push(format!("axiom is not permitted: {axiom}"));
            }
        }

        Ok(TrustEvaluation {
            policy_id: policy.policy_id.clone(),
            axiom_profile_id: profile.profile_id.clone(),
            accepted: reasons.is_empty(),
            reasons,
        })
    }
}

#[derive(Debug, Error)]
pub enum TrustPolicyError {
    #[error("axiom profile is malformed")]
    InvalidProfile,
    #[error("certification axiom profiles must reject unlisted axioms and prohibited trust paths")]
    UnsafeProfile,
    #[error("trust policy is malformed")]
    InvalidPolicy,
    #[error("certification trust policies must require exact challenges, pinned toolchains, and dependency locks")]
    UnsafePolicy,
    #[error("axiom profile identifier is not content-derived")]
    ProfileIdMismatch,
    #[error("trust policy identifier is not content-derived")]
    PolicyIdMismatch,
    #[error("trust registry root does not match its canonical content")]
    RegistryRootMismatch,
    #[error("trust registry entries are not strictly sorted by identifier")]
    RegistryNotCanonical,
    #[error("trust registry must contain at least one axiom profile and trust policy")]
    EmptyRegistry,
    #[error("trust registry contains a duplicate immutable object")]
    DuplicateObject,
    #[error("trust policy references an unknown axiom profile")]
    UnknownAxiomProfile,
    #[error("theory references an unknown trust policy")]
    UnknownTrustPolicy,
    #[error(transparent)]
    Canonical(#[from] crate::CanonicalizationError),
    #[error(transparent)]
    Id(#[from] IdError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArtifactId, ClaimId, TheoryId};

    fn fixture() -> (
        TrustPolicyRegistrySnapshot,
        TheoryManifest,
        ProofManifest,
        ProofTrustEvidence,
    ) {
        let mut profile = AxiomProfile {
            profile_id: PolicyId::derive(&"pending-profile").unwrap(),
            protocol_version: XLMP_VERSION.to_owned(),
            name: "lean-standard-v1".to_owned(),
            permitted_axioms: BTreeSet::from([
                "Classical.choice".to_owned(),
                "Quot.sound".to_owned(),
                "propext".to_owned(),
            ]),
            explicitly_forbidden_axioms: BTreeSet::from(["sorryAx".to_owned()]),
            allow_unlisted_axioms: false,
            allow_sorry: false,
            allow_unsafe_declarations: false,
            allow_native_decide: false,
            supersedes: None,
        };
        profile.profile_id = profile.derive_profile_id().unwrap();
        let mut policy = TrustPolicy {
            policy_id: PolicyId::derive(&"pending-policy").unwrap(),
            protocol_version: XLMP_VERSION.to_owned(),
            name: "gold-formal-v1".to_owned(),
            axiom_profile_id: profile.profile_id.clone(),
            permitted_checker_families: BTreeSet::from([
                CheckerFamily::LeanKernel,
                CheckerFamily::Nanoda,
            ]),
            minimum_independent_checker_families: 2,
            permitted_canonical_encodings: BTreeSet::from(["xlemma-lean-expr-v1".to_owned()]),
            require_exact_challenge: true,
            require_pinned_toolchain: true,
            require_dependency_lock: true,
            supersedes: None,
        };
        policy.policy_id = policy.derive_policy_id().unwrap();
        let mut snapshot = TrustPolicyRegistrySnapshot {
            protocol_version: XLMP_VERSION.to_owned(),
            registry_root: String::new(),
            axiom_profiles: vec![profile.clone()],
            trust_policies: vec![policy.clone()],
        };
        snapshot.registry_root = snapshot.expected_registry_root().unwrap();
        let theory_id = TheoryId::derive(&"theory").unwrap();
        let claim_id = ClaimId::from_canonical_elaborated_type(&theory_id, "True").unwrap();
        let theory = TheoryManifest {
            protocol_version: XLMP_VERSION.to_owned(),
            lean_toolchain: "leanprover/lean4:v4.33.1".to_owned(),
            dependency_merkle_root: "blake3:dependency".to_owned(),
            trust_policy_id: policy.policy_id,
            checker_policy_id: PolicyId::derive(&"checker-policy").unwrap(),
            permitted_axioms: vec!["Classical.choice".to_owned()],
            canonical_encoding: "xlemma-lean-expr-v1".to_owned(),
        };
        let proof = ProofManifest {
            protocol_version: XLMP_VERSION.to_owned(),
            claim_id: claim_id.clone(),
            canonical_proof_object: "True.intro".to_owned(),
            artifact_id: ArtifactId::derive(&"artifact").unwrap(),
            direct_dependencies: vec![],
            dependency_root: "blake3:dependency".to_owned(),
            observed_axioms: vec!["Classical.choice".to_owned()],
        };
        let evidence = ProofTrustEvidence {
            observed_axioms: BTreeSet::from(["Classical.choice".to_owned()]),
            used_sorry: false,
            used_unsafe_declarations: false,
            used_native_decide: false,
            checker_families: BTreeSet::from([CheckerFamily::LeanKernel, CheckerFamily::Nanoda]),
            exact_challenge_matched: true,
            toolchain_pinned: true,
            dependency_lock_verified: true,
        };
        (snapshot, theory, proof, evidence)
    }

    #[test]
    fn registered_policy_accepts_exact_fail_closed_evidence() {
        let (snapshot, theory, proof, evidence) = fixture();
        let registry = TrustPolicyRegistry::from_snapshot(snapshot).unwrap();
        let result = registry.evaluate(&theory, &proof, &evidence).unwrap();
        assert!(result.accepted, "{:?}", result.reasons);
    }

    #[test]
    fn unlisted_axiom_and_checker_shortfall_fail_closed() {
        let (snapshot, theory, mut proof, mut evidence) = fixture();
        let registry = TrustPolicyRegistry::from_snapshot(snapshot).unwrap();
        proof.observed_axioms = vec!["Custom.axiom".to_owned()];
        evidence.observed_axioms = BTreeSet::from(["Custom.axiom".to_owned()]);
        evidence.checker_families = BTreeSet::from([CheckerFamily::LeanKernel]);
        let result = registry.evaluate(&theory, &proof, &evidence).unwrap();
        assert!(!result.accepted);
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.contains("not permitted")));
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.contains("checker-family")));
    }

    #[test]
    fn registry_rejects_mutated_content_and_unsafe_policies() {
        let (mut snapshot, _, _, _) = fixture();
        snapshot.axiom_profiles[0].allow_sorry = true;
        assert!(matches!(
            TrustPolicyRegistry::from_snapshot(snapshot),
            Err(TrustPolicyError::RegistryRootMismatch)
        ));

        let (mut snapshot, _, _, _) = fixture();
        snapshot.axiom_profiles[0].allow_sorry = true;
        snapshot.axiom_profiles[0].profile_id =
            snapshot.axiom_profiles[0].derive_profile_id().unwrap();
        snapshot.trust_policies[0].axiom_profile_id = snapshot.axiom_profiles[0].profile_id.clone();
        snapshot.trust_policies[0].policy_id =
            snapshot.trust_policies[0].derive_policy_id().unwrap();
        snapshot.registry_root = snapshot.expected_registry_root().unwrap();
        assert!(matches!(
            TrustPolicyRegistry::from_snapshot(snapshot),
            Err(TrustPolicyError::UnsafeProfile)
        ));

        let (mut snapshot, _, _, _) = fixture();
        snapshot.trust_policies[0].require_exact_challenge = false;
        snapshot.trust_policies[0].policy_id =
            snapshot.trust_policies[0].derive_policy_id().unwrap();
        snapshot.registry_root = snapshot.expected_registry_root().unwrap();
        assert!(matches!(
            TrustPolicyRegistry::from_snapshot(snapshot),
            Err(TrustPolicyError::UnsafePolicy)
        ));

        let mut empty = TrustPolicyRegistrySnapshot {
            protocol_version: XLMP_VERSION.to_owned(),
            registry_root: String::new(),
            axiom_profiles: vec![],
            trust_policies: vec![],
        };
        empty.registry_root = empty.expected_registry_root().unwrap();
        assert!(matches!(
            TrustPolicyRegistry::from_snapshot(empty),
            Err(TrustPolicyError::EmptyRegistry)
        ));
    }
}
