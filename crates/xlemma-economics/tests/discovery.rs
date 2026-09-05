use xlemma_core::{ArtifactId, ContributionGroupId, ReceiptId, ResearcherId};
use xlemma_economics::*;

fn pilot() -> DiscoverySimulation {
    serde_json::from_str(include_str!("../../../examples/discovery/pilot.json")).unwrap()
}

fn submission(input: &mut DiscoverySimulation, index: usize) -> &mut DiscoverySubmission {
    let DiscoveryEvent::Submit { submission, .. } = &mut input.events[index] else {
        panic!("fixture")
    };
    submission
}

fn base() -> DiscoverySimulation {
    let mut input = pilot();
    input.events.truncate(6);
    input.events.push(DiscoveryEvent::Finalize { at: 210 });
    input
}

fn conserved(r: &DiscoverySimulationReport) {
    assert_eq!(
        r.declared_funding_units,
        r.administrator_fees_units
            + r.verification_spent_units
            + r.appeal_spent_units
            + r.allocated_units
            + r.retained_units
    );
    assert!(r.pending_appeal_reserved_units <= r.retained_units);
    for a in &r.allocations {
        assert_eq!(
            a.gross_units,
            a.contributor_units.values().sum::<u64>() + a.split_remainder_units
        );
    }
    assert!(r.simulation_only && !r.authenticates_evidence && !r.executes_payments);
}

#[test]
fn unsolicited_research_all_categories_and_grouping_appeal_complete() {
    let input = pilot();
    let r = simulate_discovery(&input).unwrap();
    conserved(&r);
    assert_eq!(r.allocations.len(), 7);
    assert_eq!(r.state, DiscoveryRoundState::Finalized);
    assert_eq!(r.allocated_units, 1_800_000_000);
    assert_eq!(r.verification_spent_units, 7_000_000);
    assert_eq!(r.appeal_spent_units, 2_000_000);
    assert_eq!(r.largest_donor_share_bps, 3333);
    assert_eq!(r.event_receipts.len(), input.events.len());
}

#[test]
fn pending_appeal_holds_whole_batch_and_reserves_uncompleted_review() {
    let mut input = pilot();
    input.events.truncate(8);
    let r = simulate_discovery(&input).unwrap();
    conserved(&r);
    assert_eq!(r.unresolved_appeals, 1);
    assert!(r.allocations.is_empty());
    assert_eq!(r.appeal_spent_units, 0);
    assert_eq!(r.pending_appeal_reserved_units, 2_000_000);
    input.events.push(DiscoveryEvent::Finalize { at: 210 });
    assert!(simulate_discovery(&input)
        .unwrap_err()
        .to_string()
        .contains("unresolved appeal"));
}

#[test]
fn expiry_retains_funds_and_unresolved_evidence() {
    let mut input = pilot();
    input.events.truncate(8);
    input.events.push(DiscoveryEvent::Expire { at: 301 });
    let r = simulate_discovery(&input).unwrap();
    conserved(&r);
    assert_eq!(r.state, DiscoveryRoundState::Expired);
    assert_eq!(r.unresolved_appeals, 1);
    assert_eq!(r.allocated_units, 0);
    assert_eq!(r.appeal_spent_units, 0);
}

#[test]
fn restricted_funds_pledges_and_repeated_settlements_cannot_create_funding() {
    let mut input = base();
    input.funding[0].gross_units = 1;
    input.funding[0].administrator_fee_units = 0;
    input.pledged_units = 8_000_000_000;
    assert!(simulate_discovery(&input).is_err());
    let mut input = base();
    input.funding.push(input.funding[0].clone());
    assert!(simulate_discovery(&input).is_err());
    let mut input = base();
    input
        .previously_consumed_settlements
        .insert(input.funding[0].settlement_receipt_id.clone());
    assert!(simulate_discovery(&input).is_err());
}

#[test]
fn donors_producers_and_original_assessors_cannot_review_own_work() {
    let mut input = base();
    input.funding[0].administrator_cluster = input.policy.assessors.iter().next().unwrap().clone();
    assert!(simulate_discovery(&input).is_err());
    let mut input = base();
    let assessor = input.policy.assessors.iter().next().unwrap().clone();
    submission(&mut input, 0).contributors[0].operator_cluster_id = assessor;
    assert!(simulate_discovery(&input).is_err());
    let mut input = pilot();
    let DiscoveryEvent::Resolve { reviewers, .. } = &mut input.events[8] else {
        panic!()
    };
    *reviewers = input.policy.assessors.clone();
    assert!(simulate_discovery(&input).is_err());
}

#[test]
fn reward_appeal_cannot_vote_divergence_into_validity() {
    let mut input = pilot();
    submission(&mut input, 6).declared_evidence_status = DeclaredEvidenceStatus::Divergent;
    let id = submission(&mut input, 6).submission_id.clone();
    let r = simulate_discovery(&input).unwrap();
    conserved(&r);
    assert_eq!(r.excluded_submissions[&id], "evidence_not_supported");
    assert_eq!(r.verification_spent_units, 7_000_000);
    let DiscoveryEvent::Appeal { grounds, .. } = &mut input.events[7] else {
        panic!()
    };
    *grounds = AppealGround::Evidence;
    assert!(simulate_discovery(&input).is_err());
}

#[test]
fn revalidation_remedy_leaves_evidence_inconclusive() {
    let mut input = pilot();
    let DiscoveryEvent::Resolve { remedy, .. } = &mut input.events[8] else {
        panic!()
    };
    *remedy = DiscoveryRemedy::RequireRevalidation;
    let r = simulate_discovery(&input).unwrap();
    assert_eq!(r.allocations.len(), 6);
    conserved(&r);
}

#[test]
fn raw_compute_and_rotated_identities_cannot_increase_group_award() {
    let expected = simulate_discovery(&base()).unwrap();
    let mut input = base();
    let mut duplicate = submission(&mut input, 0).clone();
    submission(&mut input, 0).reported_tokens = 9_000_000_000_000;
    submission(&mut input, 0).reported_compute_units = 9_000_000_000_000;
    duplicate.submission_id = ReceiptId::derive(&"rotated-submission").unwrap();
    duplicate.assessed_weight = 1000;
    duplicate.contributors[0].researcher_id = ResearcherId::derive(&"rotated-researcher").unwrap();
    input.events.insert(
        6,
        DiscoveryEvent::Submit {
            at: 7,
            submission: duplicate,
        },
    );
    let r = simulate_discovery(&input).unwrap();
    assert_eq!(
        serde_json::to_value(&r.allocations).unwrap(),
        serde_json::to_value(expected.allocations).unwrap()
    );
    assert_eq!(r.excluded_submissions.len(), 1);
    conserved(&r);
}

#[test]
fn exact_claim_duplicates_are_blocked_but_first_formalization_is_distinct() {
    let mut input = base();
    let original = submission(&mut input, 0).clone();
    let mut duplicate = original.clone();
    duplicate.submission_id = ReceiptId::derive(&"duplicate-claim").unwrap();
    duplicate.group_id = ContributionGroupId::derive(&"false-new-group").unwrap();
    input.events.insert(
        6,
        DiscoveryEvent::Submit {
            at: 7,
            submission: duplicate,
        },
    );
    submission(&mut input, 1).claims = original.claims;
    let r = simulate_discovery(&input).unwrap();
    assert_eq!(r.allocations.len(), 6);
    assert_eq!(r.excluded_submissions.len(), 1);
}

#[test]
fn settled_history_prevents_cross_round_replay() {
    let mut input = base();
    let s = submission(&mut input, 0).clone();
    input.previously_rewarded_groups.insert(s.group_id);
    let r = simulate_discovery(&input).unwrap();
    assert_eq!(r.allocations.len(), 5);
    assert_eq!(
        r.retained_units,
        simulate_discovery(&base()).unwrap().retained_units + 300_000_000
    );
}

#[test]
fn honest_failures_are_paid_and_zero_weights_retain_budget() {
    let mut input = base();
    for i in 0..6 {
        submission(&mut input, i).declared_evidence_status = DeclaredEvidenceStatus::Rejected;
    }
    let r = simulate_discovery(&input).unwrap();
    assert_eq!(r.verification_spent_units, 6_000_000);
    assert_eq!(r.allocated_units, 0);
    conserved(&r);
    for i in 0..6 {
        submission(&mut input, i).declared_evidence_status = DeclaredEvidenceStatus::Supported;
        submission(&mut input, i).assessed_weight = 0;
    }
    assert_eq!(simulate_discovery(&input).unwrap().allocated_units, 0);
}

#[test]
fn admission_reserves_deadlines_and_finalization_are_hard_bounds() {
    let mut input = base();
    input.policy.maximum_submissions = 5;
    assert!(simulate_discovery(&input).is_err());
    let mut input = pilot();
    input
        .policy
        .budgets
        .get_mut(&RewardCategory::Discovery)
        .unwrap()
        .verification_units = 1_000_000;
    assert!(simulate_discovery(&input).is_err());
    let mut input = base();
    for at in [199, 301] {
        input.events[6] = DiscoveryEvent::Finalize { at };
        assert!(simulate_discovery(&input).is_err());
    }
    input.events[6] = DiscoveryEvent::Finalize { at: 210 };
    input.events.push(DiscoveryEvent::Finalize { at: 211 });
    assert!(simulate_discovery(&input).is_err());
}

#[test]
fn empirical_profile_requires_uncertainty_and_evidence_presence() {
    let mut input = base();
    submission(&mut input, 3).evidence_roots.clear();
    assert!(simulate_discovery(&input).is_err());
    let mut input = base();
    input
        .policy
        .profiles
        .iter_mut()
        .find(|p| p.class == xlemma_core::VerificationProfileClass::Empirical)
        .unwrap()
        .required_evidence
        .remove(&xlemma_core::VerificationEvidenceKind::UncertaintyReport);
    assert!(simulate_discovery(&input).is_err());
}

#[test]
fn receipt_chain_commits_policy_funding_history_and_corrections() {
    let input = base();
    let r = simulate_discovery(&input).unwrap();
    assert_eq!(
        r.event_receipts,
        simulate_discovery(&input).unwrap().event_receipts
    );
    let mut changed = input.clone();
    changed.policy.name.push('x');
    assert_ne!(r.round_id, simulate_discovery(&changed).unwrap().round_id);
    let mut changed = input;
    changed.funding[0].mandate_root = ArtifactId::derive(&"changed-mandate").unwrap();
    assert_ne!(
        r.event_receipts,
        simulate_discovery(&changed).unwrap().event_receipts
    );
}

#[test]
fn repeated_appeals_unknown_fields_and_unsafe_money_are_rejected() {
    let mut input = pilot();
    input.events.insert(8, input.events[7].clone());
    assert!(simulate_discovery(&input).is_err());
    let mut input = base();
    input.funding[0].gross_units = u64::MAX;
    assert!(simulate_discovery(&input).is_err());
    let mut value = serde_json::to_value(base()).unwrap();
    value["policy"]["pay_for_tokens"] = true.into();
    assert!(serde_json::from_value::<DiscoverySimulation>(value).is_err());
}

#[test]
fn generated_properties_conserve_budget_and_ignore_partition_count() {
    let mut seed = 17u64;
    for case in 0..256 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut input = base();
        input
            .policy
            .budgets
            .get_mut(&RewardCategory::Discovery)
            .unwrap()
            .solver_units = seed % 300_000_001;
        let original = submission(&mut input, 0);
        let first = (seed % 9999 + 1) as u16;
        original.contributors[0].share_bps = first;
        original.contributors[1].share_bps = 10_000 - first;
        original.assessed_weight = seed % 1000 + 1;
        let original = original.clone();
        let mut competitor = original.clone();
        competitor.submission_id = ReceiptId::derive(&(case, "competitor")).unwrap();
        competitor.group_id = ContributionGroupId::derive(&(case, "competitor")).unwrap();
        competitor.claims = [format!("xlc:blake3:{}", "b".repeat(64)).parse().unwrap()].into();
        competitor.assessed_weight = 1000 - seed % 1000;
        input.events.insert(
            6,
            DiscoveryEvent::Submit {
                at: 7,
                submission: competitor,
            },
        );
        let expected = simulate_discovery(&input).unwrap();
        for partition in 0..case % 8 {
            let mut duplicate = original.clone();
            duplicate.submission_id = ReceiptId::derive(&(case, partition)).unwrap();
            input.events.insert(
                7 + partition,
                DiscoveryEvent::Submit {
                    at: 8 + partition as u64,
                    submission: duplicate,
                },
            );
        }
        let r = simulate_discovery(&input).unwrap();
        conserved(&r);
        assert_eq!(
            serde_json::to_value(&r.allocations).unwrap(),
            serde_json::to_value(&expected.allocations).unwrap()
        );
    }
}

#[test]
fn unrecognized_semantic_duplicates_expose_an_unsolved_trust_boundary() {
    let mut input = base();
    let mut disguised = submission(&mut input, 0).clone();
    disguised.submission_id = ReceiptId::derive(&"unrecognized-duplicate").unwrap();
    disguised.group_id = ContributionGroupId::derive(&"wrong-grouping").unwrap();
    // Synthetic fixture identity, not a source-text-derived formal ClaimID.
    disguised.claims = [format!("xlc:blake3:{}", "a".repeat(64)).parse().unwrap()].into();
    input.events.insert(
        6,
        DiscoveryEvent::Submit {
            at: 7,
            submission: disguised,
        },
    );
    let r = simulate_discovery(&input).unwrap();
    assert_eq!(r.allocations.len(), 7);
    assert!(r.limitations.iter().any(|s| s.contains("evade grouping")));
}

#[test]
fn attribution_correction_uses_a_new_manifest_and_checks_new_conflicts() {
    let mut input = pilot();
    let mut shares = submission(&mut input, 6).contributors.clone();
    shares[0].researcher_id = ResearcherId::derive(&"corrected-originator").unwrap();
    let DiscoveryEvent::Appeal { grounds, .. } = &mut input.events[7] else {
        panic!()
    };
    *grounds = AppealGround::Attribution;
    let DiscoveryEvent::Resolve { remedy, .. } = &mut input.events[8] else {
        panic!()
    };
    *remedy = DiscoveryRemedy::CorrectContributors {
        contributors: shares.clone(),
        manifest_root: ArtifactId::derive(&"corrected-manifest").unwrap(),
    };
    let r = simulate_discovery(&input).unwrap();
    conserved(&r);
    let reviewer = input.policy.appeal_reviewers.iter().next().unwrap().clone();
    let DiscoveryEvent::Resolve {
        remedy: DiscoveryRemedy::CorrectContributors { contributors, .. },
        ..
    } = &mut input.events[8]
    else {
        panic!()
    };
    contributors[0].operator_cluster_id = reviewer;
    assert!(simulate_discovery(&input).is_err());
}

#[test]
fn no_round_may_shorten_profile_challenge_windows() {
    let mut input = base();
    input.policy.profiles[0].challenge_window_seconds = 101;
    assert!(simulate_discovery(&input)
        .unwrap_err()
        .to_string()
        .contains("challenge window"));
}

#[test]
fn registered_replication_payment_is_outcome_neutral() {
    let mut input = base();
    let expected = simulate_discovery(&input).unwrap();
    submission(&mut input, 3)
        .registered_study
        .as_mut()
        .unwrap()
        .outcome = StudyOutcome::Null;
    assert_eq!(
        serde_json::to_value(expected.allocations).unwrap(),
        serde_json::to_value(simulate_discovery(&input).unwrap().allocations).unwrap()
    );
    submission(&mut input, 3)
        .registered_study
        .as_mut()
        .unwrap()
        .registered_at = 2;
    assert!(simulate_discovery(&input).is_err());
}
