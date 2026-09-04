use chrono::{DateTime, Utc};
use std::collections::BTreeSet;
use thiserror::Error;
use xlemma_core::{
    canonical_json_hash, derive_eligible_set_root, CommitteeSelection, CommitteeSortitionRequest,
    CredentialTier, EligibleNode, NodeRole, OperatorClusterId, OperatorId, SortitionMember,
    VerifiedUserId,
};

/// Reference-conformance bounds keep adversarial sortition inputs from
/// triggering unbounded combinatorial search. Larger deployments can shard an
/// eligible set before committing its root, but MUST publish that policy.
pub const MAX_COMMITTEE_SLOTS: usize = 32;
pub const MAX_ELIGIBLE_NODES: usize = 1_024;
pub const MAX_SORTITION_SEARCH_STATES: usize = 1_000_000;

/// Admission is cryptographic and registry-backed, not a structural string
/// check. The consensus crate deliberately requires a verifier supplied by the
/// deployment instead of shipping a permissive production default.
pub trait CommitteeAdmissionVerifier {
    fn verify_candidate(
        &self,
        node: &EligibleNode,
        requirement: &xlemma_core::CommitteeRequirement,
        selected_at: DateTime<Utc>,
    ) -> bool;
}

#[derive(Debug, Error)]
pub enum CommitteeError {
    #[error("sortition randomness reveal must not be empty")]
    EmptyRandomness,
    #[error("sortition randomness reveal does not match its commitment")]
    RandomnessMismatch,
    #[error("eligible node set does not match the request commitment")]
    EligibleSetMismatch,
    #[error("committee requirements or diversity constraints are invalid")]
    InvalidRequirements,
    #[error("committee contains duplicate role requirements")]
    DuplicateRole,
    #[error("eligible set contains a duplicate NodeID")]
    DuplicateEligibleNode,
    #[error(
        "no committee satisfies credential, role, bond, reputation, checker, and independence constraints"
    )]
    NoIndependentCommittee,
    #[error("committee search exceeded the deterministic conformance bound")]
    SearchLimitExceeded,
    #[error("published committee selection does not reproduce exactly")]
    SelectionMismatch,
    #[error(transparent)]
    Canonicalization(#[from] xlemma_core::CanonicalizationError),
    #[error(transparent)]
    Identifier(#[from] xlemma_core::IdError),
}

#[derive(Clone, Copy)]
struct Slot {
    role: NodeRole,
    ordinal: u16,
    requirement_index: usize,
}

#[derive(Clone, Copy)]
struct RankedCandidate<'a> {
    rank: [u8; 32],
    node: &'a EligibleNode,
}

/// Commits a future randomness reveal to a specific sortition domain.
pub fn randomness_commitment(seed: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"xlemma-sortition-randomness-v1\0");
    hasher.update(&(seed.len() as u64).to_le_bytes());
    hasher.update(seed);
    format!("blake3:{}", hasher.finalize().to_hex())
}

/// Canonical commitment to every eligible-node record considered by sortition.
pub fn eligible_set_root(nodes: &[EligibleNode]) -> Result<String, CommitteeError> {
    Ok(derive_eligible_set_root(nodes)?)
}

/// Runs deterministic, auditable, stake-capped committee sortition.
///
/// Bond and reputation are eligibility gates only. Eligible candidates are
/// ranked by public randomness; neither money nor a composite reputation score
/// weights mathematical authority. Deterministic backtracking prevents greedy
/// role assignment from hiding a valid independent committee.
pub fn select_committee(
    request: &CommitteeSortitionRequest,
    revealed_seed: &[u8],
    nodes: &[EligibleNode],
    admission_verifier: &impl CommitteeAdmissionVerifier,
    selected_at: DateTime<Utc>,
) -> Result<CommitteeSelection, CommitteeError> {
    validate_request(request)?;
    if nodes.len() > MAX_ELIGIBLE_NODES {
        return Err(CommitteeError::InvalidRequirements);
    }
    request.sortition_id.validate()?;
    request.job_id.validate()?;
    request.policy_id.validate()?;
    for operator_cluster_id in &request.excluded_operator_clusters {
        operator_cluster_id.validate()?;
    }
    let mut node_ids = BTreeSet::new();
    for node in nodes {
        node.node_id.validate()?;
        node.operator_id.validate()?;
        node.operator_cluster_id.validate()?;
        node.advertisement_id.validate()?;
        node.bond_id.validate()?;
        node.reputation_snapshot_id.validate()?;
        if !node_ids.insert(&node.node_id) {
            return Err(CommitteeError::DuplicateEligibleNode);
        }
    }
    if selected_at < request.requested_at {
        return Err(CommitteeError::InvalidRequirements);
    }
    if revealed_seed.is_empty() {
        return Err(CommitteeError::EmptyRandomness);
    }
    if randomness_commitment(revealed_seed) != request.randomness.seed_commitment {
        return Err(CommitteeError::RandomnessMismatch);
    }
    if eligible_set_root(nodes)? != request.eligible_set_root {
        return Err(CommitteeError::EligibleSetMismatch);
    }

    let mut requirement_indices = (0..request.requirements.len()).collect::<Vec<_>>();
    requirement_indices.sort_by_key(|index| role_label(request.requirements[*index].role));

    let mut seen_roles = BTreeSet::new();
    let mut slots = Vec::new();
    for requirement_index in requirement_indices {
        let requirement = &request.requirements[requirement_index];
        if !seen_roles.insert(requirement.role) {
            return Err(CommitteeError::DuplicateRole);
        }
        for ordinal in 0..requirement.count {
            slots.push(Slot {
                role: requirement.role,
                ordinal,
                requirement_index,
            });
        }
    }

    let ranked_by_slot = slots
        .iter()
        .map(|slot| {
            let requirement = &request.requirements[slot.requirement_index];
            let mut candidates = nodes
                .iter()
                .filter(|node| {
                    node_is_eligible(request, requirement, node, admission_verifier, selected_at)
                })
                .map(|node| RankedCandidate {
                    rank: rank_candidate(request, revealed_seed, *slot, node),
                    node,
                })
                .collect::<Vec<_>>();
            candidates.sort_by(|left, right| {
                left.rank
                    .cmp(&right.rank)
                    .then_with(|| left.node.node_id.cmp(&right.node.node_id))
            });
            candidates
        })
        .collect::<Vec<_>>();

    if ranked_by_slot.iter().any(Vec::is_empty) {
        return Err(CommitteeError::NoIndependentCommittee);
    }

    let mut search = SearchState::default();
    match choose_independent_committee(
        &slots,
        &ranked_by_slot,
        0,
        request.minimum_distinct_providers,
        request.minimum_distinct_regions,
        &mut search,
    ) {
        SearchResult::Found => {}
        SearchResult::NotFound => return Err(CommitteeError::NoIndependentCommittee),
        SearchResult::LimitExceeded => return Err(CommitteeError::SearchLimitExceeded),
    }

    let members = search
        .chosen
        .into_iter()
        .map(|(slot, candidate)| {
            Ok(SortitionMember {
                role: slot.role,
                slot: slot.ordinal,
                node_id: candidate.node.node_id.clone(),
                verified_user_id: candidate
                    .node
                    .credential_chain
                    .user
                    .verified_user_id
                    .clone(),
                operator_id: candidate.node.operator_id.clone(),
                operator_cluster_id: candidate.node.operator_cluster_id.clone(),
                user_credential_id: candidate.node.credential_chain.user.credential_id.clone(),
                operator_credential_id: candidate
                    .node
                    .credential_chain
                    .operator
                    .credential_id
                    .clone(),
                node_credential_id: candidate.node.credential_chain.node.credential_id.clone(),
                credential_tier: candidate.node.credential_chain.user.tier,
                credential_chain_root: candidate.node.credential_chain.derive_chain_root()?,
                advertisement_id: candidate.node.advertisement_id.clone(),
                bond_id: candidate.node.bond_id.clone(),
                reputation_snapshot_id: candidate.node.reputation_snapshot_id.clone(),
                infrastructure_provider: candidate.node.infrastructure_provider.clone(),
                region: candidate.node.region.clone(),
                rank_hash: format!(
                    "blake3:{}",
                    blake3::Hash::from_bytes(candidate.rank).to_hex()
                ),
            })
        })
        .collect::<Result<Vec<_>, CommitteeError>>()?;

    let selection_digest = canonical_json_hash(
        "xlemma-committee-selection-v1",
        &(
            &request.sortition_id,
            &request.job_id,
            &request.policy_id,
            &request.randomness.seed_commitment,
            &request.eligible_set_root,
            &members,
            selected_at,
        ),
    )?;

    Ok(CommitteeSelection {
        sortition_id: request.sortition_id.clone(),
        job_id: request.job_id.clone(),
        policy_id: request.policy_id.clone(),
        randomness_commitment: request.randomness.seed_commitment.clone(),
        eligible_set_root: request.eligible_set_root.clone(),
        members,
        selection_root: format!(
            "blake3:{}",
            blake3::Hash::from_bytes(selection_digest).to_hex()
        ),
        selected_at,
    })
}

pub fn verify_committee_selection(
    request: &CommitteeSortitionRequest,
    revealed_seed: &[u8],
    nodes: &[EligibleNode],
    admission_verifier: &impl CommitteeAdmissionVerifier,
    selection: &CommitteeSelection,
) -> Result<(), CommitteeError> {
    let reproduced = select_committee(
        request,
        revealed_seed,
        nodes,
        admission_verifier,
        selection.selected_at,
    )?;
    if &reproduced == selection {
        Ok(())
    } else {
        Err(CommitteeError::SelectionMismatch)
    }
}

fn validate_request(request: &CommitteeSortitionRequest) -> Result<(), CommitteeError> {
    if request.requirements.is_empty()
        || request.randomness.source.trim().is_empty()
        || request.randomness.round == 0
        || request.randomness.seed_commitment.trim().is_empty()
        || request.randomness.proof_reference.trim().is_empty()
        || request.eligible_set_root.trim().is_empty()
        || request.requirements.iter().any(|requirement| {
            requirement.count == 0
                || requirement.minimum_bond.units == 0
                || requirement.minimum_bond.asset.trim().is_empty()
                || (matches!(
                    requirement.role,
                    NodeRole::OfficialKernelChecker | NodeRole::IndependentChecker
                ) && requirement.required_checker_families.is_empty())
                || requirement.minimum_credential_tier < CredentialTier::V2VerifiedOperator
                || requirement.maximum_status_age_seconds == 0
        })
    {
        return Err(CommitteeError::InvalidRequirements);
    }
    let slots = request
        .requirements
        .iter()
        .map(|requirement| usize::from(requirement.count))
        .sum::<usize>();
    if slots > MAX_COMMITTEE_SLOTS
        || request.minimum_distinct_providers == 0
        || request.minimum_distinct_regions == 0
        || usize::from(request.minimum_distinct_providers) > slots
        || usize::from(request.minimum_distinct_regions) > slots
    {
        return Err(CommitteeError::InvalidRequirements);
    }
    Ok(())
}

fn node_is_eligible(
    request: &CommitteeSortitionRequest,
    requirement: &xlemma_core::CommitteeRequirement,
    node: &EligibleNode,
    admission_verifier: &impl CommitteeAdmissionVerifier,
    selected_at: DateTime<Utc>,
) -> bool {
    node.active
        && node.operator_id == node.credential_chain.operator.operator_id
        && node.roles.contains(&requirement.role)
        && !request
            .excluded_operator_clusters
            .contains(&node.operator_cluster_id)
        && !node.infrastructure_provider.trim().is_empty()
        && !node.region.trim().is_empty()
        && node
            .active_bond
            .ensure_compatible(&requirement.minimum_bond)
            .is_ok()
        && node.active_bond.units >= requirement.minimum_bond.units
        && node.reputation.meets(&requirement.reputation_requirements)
        && requirement
            .required_checker_families
            .is_subset(&node.checker_families)
        && selected_at
            .signed_duration_since(node.credential_chain.status.checked_at)
            .num_seconds()
            <= i64::try_from(requirement.maximum_status_age_seconds).unwrap_or(i64::MAX)
        && node
            .credential_chain
            .validate_for(
                &node.node_id,
                &node.operator_cluster_id,
                requirement.role,
                requirement.minimum_credential_tier,
                &requirement.required_qualifications,
                selected_at,
            )
            .is_ok()
        && admission_verifier.verify_candidate(node, requirement, selected_at)
}

fn rank_candidate(
    request: &CommitteeSortitionRequest,
    revealed_seed: &[u8],
    slot: Slot,
    node: &EligibleNode,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"xlemma-committee-sortition-v2\0");
    update_field(&mut hasher, revealed_seed);
    update_field(&mut hasher, request.sortition_id.as_str().as_bytes());
    update_field(&mut hasher, request.job_id.as_str().as_bytes());
    update_field(&mut hasher, request.policy_id.as_str().as_bytes());
    update_field(&mut hasher, &request.epoch.to_le_bytes());
    update_field(&mut hasher, role_label(slot.role).as_bytes());
    update_field(&mut hasher, &slot.ordinal.to_le_bytes());
    update_field(&mut hasher, node.node_id.as_str().as_bytes());
    update_field(&mut hasher, node.operator_id.as_str().as_bytes());
    update_field(
        &mut hasher,
        node.credential_chain
            .user
            .verified_user_id
            .as_str()
            .as_bytes(),
    );
    update_field(&mut hasher, request.eligible_set_root.as_bytes());
    *hasher.finalize().as_bytes()
}

fn update_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchResult {
    Found,
    NotFound,
    LimitExceeded,
}

#[derive(Default)]
struct SearchState<'a> {
    operator_clusters: BTreeSet<OperatorClusterId>,
    operators: BTreeSet<OperatorId>,
    verified_users: BTreeSet<VerifiedUserId>,
    chosen: Vec<(Slot, RankedCandidate<'a>)>,
    explored_states: usize,
}

fn choose_independent_committee<'a>(
    slots: &[Slot],
    candidates: &[Vec<RankedCandidate<'a>>],
    index: usize,
    minimum_distinct_providers: u16,
    minimum_distinct_regions: u16,
    search: &mut SearchState<'a>,
) -> SearchResult {
    if search.explored_states >= MAX_SORTITION_SEARCH_STATES {
        return SearchResult::LimitExceeded;
    }
    search.explored_states += 1;
    if index == slots.len() {
        let providers = search
            .chosen
            .iter()
            .map(|(_, candidate)| candidate.node.infrastructure_provider.as_str())
            .collect::<BTreeSet<_>>();
        let regions = search
            .chosen
            .iter()
            .map(|(_, candidate)| candidate.node.region.as_str())
            .collect::<BTreeSet<_>>();
        return if providers.len() >= usize::from(minimum_distinct_providers)
            && regions.len() >= usize::from(minimum_distinct_regions)
        {
            SearchResult::Found
        } else {
            SearchResult::NotFound
        };
    }

    for candidate in &candidates[index] {
        if !search
            .operator_clusters
            .insert(candidate.node.operator_cluster_id.clone())
        {
            continue;
        }
        if !search.operators.insert(candidate.node.operator_id.clone()) {
            let removed = search
                .operator_clusters
                .remove(&candidate.node.operator_cluster_id);
            debug_assert!(removed);
            continue;
        }
        let verified_user_id = &candidate.node.credential_chain.user.verified_user_id;
        if !search.verified_users.insert(verified_user_id.clone()) {
            let removed = search.operators.remove(&candidate.node.operator_id);
            debug_assert!(removed);
            let removed = search
                .operator_clusters
                .remove(&candidate.node.operator_cluster_id);
            debug_assert!(removed);
            continue;
        }
        search.chosen.push((slots[index], *candidate));
        let result = choose_independent_committee(
            slots,
            candidates,
            index + 1,
            minimum_distinct_providers,
            minimum_distinct_regions,
            search,
        );
        if result == SearchResult::Found {
            return result;
        }
        let removed = search.chosen.pop();
        debug_assert!(removed.is_some());
        let removed = search.verified_users.remove(verified_user_id);
        debug_assert!(removed);
        let removed = search.operators.remove(&candidate.node.operator_id);
        debug_assert!(removed);
        let removed = search
            .operator_clusters
            .remove(&candidate.node.operator_cluster_id);
        debug_assert!(removed);
        if result == SearchResult::LimitExceeded {
            return result;
        }
    }
    SearchResult::NotFound
}

fn role_label(role: NodeRole) -> &'static str {
    match role {
        NodeRole::Researcher => "researcher",
        NodeRole::ResearchProver => "research_prover",
        NodeRole::LeanBuilder => "lean_builder",
        NodeRole::OfficialKernelChecker => "official_kernel_checker",
        NodeRole::IndependentChecker => "independent_checker",
        NodeRole::NoveltyReviewer => "novelty_reviewer",
        NodeRole::SignificanceReviewer => "significance_reviewer",
        NodeRole::Challenger => "challenger",
        NodeRole::StorageProvider => "storage_provider",
        NodeRole::Indexer => "indexer",
        NodeRole::PaymentFacilitator => "payment_facilitator",
        NodeRole::CertificateFinalizer => "certificate_finalizer",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use xlemma_core::{
        AdvertisementId, Amount, BondId, CheckerFamily, CommitteeRequirement,
        CredentialStatusProof, JobId, NodeCredential, NodeCredentialChain, NodeCredentialId,
        NodeReputationVector, OperatorCredential, OperatorCredentialId, PolicyId, RandomnessBeacon,
        ReputationId, ReputationMetric, ReputationRequirement, ReputationRequirements, SortitionId,
        UserCredential, UserCredentialId,
    };

    struct AcceptTestAdmission;

    impl CommitteeAdmissionVerifier for AcceptTestAdmission {
        fn verify_candidate(
            &self,
            _node: &EligibleNode,
            _requirement: &CommitteeRequirement,
            _selected_at: DateTime<Utc>,
        ) -> bool {
            true
        }
    }

    fn metric(score_bps: u16) -> ReputationMetric {
        ReputationMetric {
            score_bps,
            sample_size: 100,
            evidence_root: "blake3:evidence".into(),
        }
    }

    fn reputation(formal_accuracy_bps: u16) -> NodeReputationVector {
        NodeReputationVector {
            formal_accuracy: metric(formal_accuracy_bps),
            availability: metric(9_900),
            latency: metric(9_500),
            novelty_calibration: metric(9_000),
            challenge_quality: metric(9_000),
            independence: metric(9_900),
            storage_quality: metric(9_400),
            integrity: metric(9_900),
        }
    }

    fn credential_chain(
        label: &str,
        node_id: &xlemma_core::NodeId,
        operator_cluster_id: &OperatorClusterId,
        roles: &BTreeSet<NodeRole>,
        now: DateTime<Utc>,
    ) -> NodeCredentialChain {
        let verified_user_id = VerifiedUserId::derive(&format!("user-{label}")).unwrap();
        let operator_id = OperatorId::derive(&format!("operator-{label}")).unwrap();
        let mut user = UserCredential {
            credential_id: UserCredentialId::derive(&"pending").unwrap(),
            verified_user_id: verified_user_id.clone(),
            researcher_id: None,
            public_subject: format!("did:key:user-{label}"),
            tier: CredentialTier::V2VerifiedOperator,
            issuer: "did:web:credential-issuer.example".into(),
            uniqueness_commitment: format!("blake3:uniqueness-{label}"),
            qualifications: BTreeSet::from(["lean-kernel".into()]),
            disclosure_policy: "pseudonymous-v1".into(),
            issued_at: now - Duration::days(1),
            expires_at: now + Duration::days(30),
            evidence_root: format!("blake3:user-evidence-{label}"),
            issuer_signature: "issuer-signature".into(),
        };
        user.credential_id = user.derive_credential_id().unwrap();
        let mut operator = OperatorCredential {
            credential_id: OperatorCredentialId::derive(&"pending").unwrap(),
            operator_id: operator_id.clone(),
            verified_user_id,
            user_credential_id: user.credential_id.clone(),
            operator_cluster_id: operator_cluster_id.clone(),
            authorized_roles: roles.clone(),
            qualifications: BTreeSet::from(["lean-kernel".into()]),
            jurisdiction_class: "privacy-preserving-verified".into(),
            issued_at: user.issued_at,
            expires_at: user.expires_at,
            evidence_root: format!("blake3:operator-evidence-{label}"),
            holder_delegation_signature: "holder-delegation-signature".into(),
            issuer_signature: "issuer-signature".into(),
        };
        operator.credential_id = operator.derive_credential_id().unwrap();
        let mut node = NodeCredential {
            credential_id: NodeCredentialId::derive(&"pending").unwrap(),
            node_id: node_id.clone(),
            operator_id,
            operator_credential_id: operator.credential_id.clone(),
            operator_cluster_id: operator_cluster_id.clone(),
            node_public_key: format!("did:key:node-{label}"),
            authorized_roles: roles.clone(),
            hardware_attestation_root: None,
            issued_at: operator.issued_at,
            expires_at: operator.expires_at,
            evidence_root: format!("blake3:node-evidence-{label}"),
            operator_delegation_signature: "operator-delegation-signature".into(),
        };
        node.credential_id = node.derive_credential_id().unwrap();
        let status = CredentialStatusProof {
            user_credential_id: user.credential_id.clone(),
            operator_credential_id: operator.credential_id.clone(),
            node_credential_id: node.credential_id.clone(),
            revocation_registry_root: "blake3:revocation-registry-root".into(),
            checked_at: now - Duration::minutes(1),
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

    fn eligible_node(
        label: &str,
        operator: &str,
        roles: BTreeSet<NodeRole>,
        provider: &str,
        region: &str,
    ) -> EligibleNode {
        let now = Utc::now();
        let node_id = xlemma_core::NodeId::derive(&label).unwrap();
        let operator_cluster_id = OperatorClusterId::derive(&operator).unwrap();
        let credential_chain = credential_chain(label, &node_id, &operator_cluster_id, &roles, now);
        EligibleNode {
            node_id,
            operator_id: credential_chain.operator.operator_id.clone(),
            operator_cluster_id,
            credential_chain,
            advertisement_id: AdvertisementId::derive(&label).unwrap(),
            roles,
            checker_families: BTreeSet::from([CheckerFamily::LeanKernel, CheckerFamily::Nanoda]),
            bond_id: BondId::derive(&label).unwrap(),
            active_bond: Amount::new(1_000, "USDC", 6),
            reputation_snapshot_id: ReputationId::derive(&label).unwrap(),
            reputation: reputation(9_800),
            infrastructure_provider: provider.into(),
            region: region.into(),
            active: true,
        }
    }

    fn requirement(role: NodeRole, count: u16) -> CommitteeRequirement {
        CommitteeRequirement {
            role,
            count,
            minimum_bond: Amount::new(100, "USDC", 6),
            reputation_requirements: ReputationRequirements {
                formal_accuracy: Some(ReputationRequirement {
                    minimum_score_bps: 9_000,
                    minimum_sample_size: 10,
                }),
                ..ReputationRequirements::default()
            },
            required_checker_families: match role {
                NodeRole::OfficialKernelChecker => BTreeSet::from([CheckerFamily::LeanKernel]),
                NodeRole::IndependentChecker => BTreeSet::from([CheckerFamily::Nanoda]),
                _ => BTreeSet::new(),
            },
            minimum_credential_tier: CredentialTier::V2VerifiedOperator,
            maximum_status_age_seconds: 3_600,
            required_qualifications: BTreeSet::new(),
        }
    }

    fn request(
        nodes: &[EligibleNode],
        requirements: Vec<CommitteeRequirement>,
    ) -> CommitteeSortitionRequest {
        let seed = b"future-beacon-reveal";
        CommitteeSortitionRequest {
            sortition_id: SortitionId::derive(&"sortition").unwrap(),
            job_id: JobId::derive(&"job").unwrap(),
            policy_id: PolicyId::derive(&"policy").unwrap(),
            epoch: 7,
            eligible_set_root: eligible_set_root(nodes).unwrap(),
            randomness: RandomnessBeacon {
                source: "drand".into(),
                round: 42,
                seed_commitment: randomness_commitment(seed),
                proof_reference: "blake3:beacon-proof".into(),
            },
            requirements,
            minimum_distinct_providers: 2,
            minimum_distinct_regions: 2,
            excluded_operator_clusters: BTreeSet::new(),
            requested_at: Utc::now(),
        }
    }

    #[test]
    fn selection_is_reproducible_and_operator_independent() {
        let role = NodeRole::OfficialKernelChecker;
        let nodes = vec![
            eligible_node("a", "op-a", BTreeSet::from([role]), "p-a", "r-a"),
            eligible_node("b", "op-b", BTreeSet::from([role]), "p-b", "r-b"),
            eligible_node("c", "op-c", BTreeSet::from([role]), "p-c", "r-c"),
        ];
        let request = request(&nodes, vec![requirement(role, 2)]);
        let selected_at = Utc::now();
        let left = select_committee(
            &request,
            b"future-beacon-reveal",
            &nodes,
            &AcceptTestAdmission,
            selected_at,
        )
        .unwrap();
        let right = select_committee(
            &request,
            b"future-beacon-reveal",
            &nodes,
            &AcceptTestAdmission,
            selected_at,
        )
        .unwrap();
        assert_eq!(left, right);
        assert_eq!(left.members.len(), 2);
        assert_ne!(
            left.members[0].operator_cluster_id,
            left.members[1].operator_cluster_id
        );
        verify_committee_selection(
            &request,
            b"future-beacon-reveal",
            &nodes,
            &AcceptTestAdmission,
            &left,
        )
        .unwrap();
    }

    #[test]
    fn one_operator_cannot_fill_multiple_roles() {
        let kernel = NodeRole::OfficialKernelChecker;
        let independent = NodeRole::IndependentChecker;
        let nodes = vec![eligible_node(
            "a",
            "same-operator",
            BTreeSet::from([kernel, independent]),
            "p-a",
            "r-a",
        )];
        let mut request = request(
            &nodes,
            vec![requirement(kernel, 1), requirement(independent, 1)],
        );
        request.minimum_distinct_providers = 1;
        request.minimum_distinct_regions = 1;
        assert!(matches!(
            select_committee(
                &request,
                b"future-beacon-reveal",
                &nodes,
                &AcceptTestAdmission,
                Utc::now()
            ),
            Err(CommitteeError::NoIndependentCommittee)
        ));
    }

    #[test]
    fn one_verified_participant_cannot_gain_independence_with_multiple_operators() {
        let role = NodeRole::OfficialKernelChecker;
        let mut nodes = vec![
            eligible_node("a", "cluster-a", BTreeSet::from([role]), "p-a", "r-a"),
            eligible_node("b", "cluster-b", BTreeSet::from([role]), "p-b", "r-b"),
        ];
        let shared_user = nodes[0].credential_chain.user.clone();
        let second = &mut nodes[1].credential_chain;
        second.user = shared_user;
        second.operator.verified_user_id = second.user.verified_user_id.clone();
        second.operator.user_credential_id = second.user.credential_id.clone();
        second.operator.issued_at = second.user.issued_at;
        second.operator.expires_at = second.user.expires_at;
        second.operator.credential_id = second.operator.derive_credential_id().unwrap();
        second.node.operator_credential_id = second.operator.credential_id.clone();
        second.node.issued_at = second.operator.issued_at;
        second.node.expires_at = second.operator.expires_at;
        second.node.credential_id = second.node.derive_credential_id().unwrap();
        second.status.user_credential_id = second.user.credential_id.clone();
        second.status.operator_credential_id = second.operator.credential_id.clone();
        second.status.node_credential_id = second.node.credential_id.clone();

        let request = request(&nodes, vec![requirement(role, 2)]);
        assert!(matches!(
            select_committee(
                &request,
                b"future-beacon-reveal",
                &nodes,
                &AcceptTestAdmission,
                Utc::now()
            ),
            Err(CommitteeError::NoIndependentCommittee)
        ));
    }

    #[test]
    fn money_and_reputation_are_eligibility_not_rank_weight() {
        let role = NodeRole::OfficialKernelChecker;
        let mut low_accuracy =
            eligible_node("wealthy", "op-rich", BTreeSet::from([role]), "p-a", "r-a");
        low_accuracy.active_bond.units = 1_000_000;
        low_accuracy.reputation = reputation(8_000);
        let qualified = eligible_node(
            "qualified",
            "op-qualified",
            BTreeSet::from([role]),
            "p-b",
            "r-b",
        );
        let nodes = vec![low_accuracy, qualified];
        let mut request = request(&nodes, vec![requirement(role, 1)]);
        request.minimum_distinct_providers = 1;
        request.minimum_distinct_regions = 1;
        let selection = select_committee(
            &request,
            b"future-beacon-reveal",
            &nodes,
            &AcceptTestAdmission,
            Utc::now(),
        )
        .unwrap();
        assert_eq!(selection.members[0].node_id, nodes[1].node_id);
    }

    #[test]
    fn randomness_and_eligible_set_commitments_fail_closed() {
        let role = NodeRole::OfficialKernelChecker;
        let nodes = vec![eligible_node(
            "a",
            "op-a",
            BTreeSet::from([role]),
            "p-a",
            "r-a",
        )];
        let mut sortition_request = request(&nodes, vec![requirement(role, 1)]);
        sortition_request.minimum_distinct_providers = 1;
        sortition_request.minimum_distinct_regions = 1;
        assert!(matches!(
            select_committee(
                &sortition_request,
                b"wrong",
                &nodes,
                &AcceptTestAdmission,
                Utc::now()
            ),
            Err(CommitteeError::RandomnessMismatch)
        ));

        let mut mutated_nodes = nodes.clone();
        mutated_nodes[0].active = false;
        assert!(matches!(
            select_committee(
                &sortition_request,
                b"future-beacon-reveal",
                &mutated_nodes,
                &AcceptTestAdmission,
                Utc::now()
            ),
            Err(CommitteeError::EligibleSetMismatch)
        ));

        let duplicate_nodes = vec![nodes[0].clone(), nodes[0].clone()];
        let mut duplicate_request = request(&duplicate_nodes, vec![requirement(role, 1)]);
        duplicate_request.minimum_distinct_providers = 1;
        duplicate_request.minimum_distinct_regions = 1;
        assert!(matches!(
            select_committee(
                &duplicate_request,
                b"future-beacon-reveal",
                &duplicate_nodes,
                &AcceptTestAdmission,
                Utc::now()
            ),
            Err(CommitteeError::DuplicateEligibleNode)
        ));
    }

    #[test]
    fn stale_non_revocation_status_cannot_enter_consensus() {
        let role = NodeRole::OfficialKernelChecker;
        let mut nodes = vec![eligible_node(
            "stale",
            "operator-stale",
            BTreeSet::from([role]),
            "p-a",
            "r-a",
        )];
        nodes[0].credential_chain.status.checked_at = Utc::now() - Duration::hours(2);
        nodes[0].credential_chain.status.valid_until = Utc::now() + Duration::hours(1);
        let mut sortition_request = request(&nodes, vec![requirement(role, 1)]);
        sortition_request.minimum_distinct_providers = 1;
        sortition_request.minimum_distinct_regions = 1;
        assert!(matches!(
            select_committee(
                &sortition_request,
                b"future-beacon-reveal",
                &nodes,
                &AcceptTestAdmission,
                Utc::now()
            ),
            Err(CommitteeError::NoIndependentCommittee)
        ));
    }

    #[test]
    fn published_sortition_vector_reproduces_exactly() {
        let mut nodes: Vec<EligibleNode> = serde_json::from_str(include_str!(
            "../../../examples/node-network/eligible-nodes.json"
        ))
        .unwrap();
        let request: CommitteeSortitionRequest = serde_json::from_str(include_str!(
            "../../../examples/node-network/sortition-request.json"
        ))
        .unwrap();
        let expected: CommitteeSelection = serde_json::from_str(include_str!(
            "../../../examples/node-network/committee-selection.json"
        ))
        .unwrap();
        assert_eq!(
            eligible_set_root(&nodes).unwrap(),
            request.eligible_set_root
        );
        let selection = select_committee(
            &request,
            b"future-beacon-reveal",
            &nodes,
            &AcceptTestAdmission,
            expected.selected_at,
        )
        .unwrap();
        assert_eq!(selection, expected);

        nodes.reverse();
        verify_committee_selection(
            &request,
            b"future-beacon-reveal",
            &nodes,
            &AcceptTestAdmission,
            &selection,
        )
        .unwrap();
    }
}
