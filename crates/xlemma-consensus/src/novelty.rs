use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xlemma_core::{NoveltyDecision, NoveltyReviewReceipt, OperatorClusterId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewerWeight {
    pub operator_cluster_id: OperatorClusterId,
    pub calibration: f64,
    pub domain_score: f64,
    pub independence: f64,
    pub evidence_quality: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoveltyPolicy {
    pub prior_probability: f64,
    pub material_novelty_threshold: f64,
    pub known_equivalent_threshold: f64,
    pub minimum_prior_art_coverage: f64,
    pub maximum_reviewer_weight: f64,
    pub minimum_independent_operators: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoveltyOutcome {
    pub posterior_material_novelty: f64,
    pub posterior_known_equivalent: f64,
    pub weighted_prior_art_coverage: f64,
    pub decision: NoveltyDecision,
    pub reviewer_count: usize,
    pub independent_operator_count: usize,
}

/// Verifies the reviewer signature and the reputation/calibration evidence
/// that produced its weight. Deployments must not aggregate bare score data.
pub trait NoveltyEvidenceVerifier {
    fn verify(&self, review: &NoveltyReviewReceipt, weight: &ReviewerWeight) -> bool;
}

#[derive(Debug, Error)]
pub enum NoveltyError {
    #[error("no novelty reviews supplied")]
    Empty,
    #[error("missing reviewer weight for operator {0}")]
    MissingWeight(String),
    #[error("invalid probability or weight")]
    InvalidValue,
    #[error("insufficient independent reviewer operators")]
    InsufficientIndependence,
    #[error("novelty reviews refer to different claims")]
    MixedClaims,
    #[error("duplicate novelty receipt, reviewer node, or operator cluster")]
    DuplicateReviewer,
    #[error("novelty receipt signature or evidence binding is empty")]
    InvalidReceipt,
    #[error("novelty receipt signature or reviewer weight proof is invalid")]
    InvalidProof,
}

pub fn aggregate_novelty(
    policy: &NoveltyPolicy,
    reviews: &[NoveltyReviewReceipt],
    weights: &[ReviewerWeight],
    verifier: &impl NoveltyEvidenceVerifier,
) -> Result<NoveltyOutcome, NoveltyError> {
    if reviews.is_empty() {
        return Err(NoveltyError::Empty);
    }

    for value in [
        policy.prior_probability,
        policy.material_novelty_threshold,
        policy.known_equivalent_threshold,
        policy.minimum_prior_art_coverage,
        policy.maximum_reviewer_weight,
    ] {
        validate_probability(value)?;
    }
    if policy.maximum_reviewer_weight <= 0.0 || policy.minimum_independent_operators == 0 {
        return Err(NoveltyError::InvalidValue);
    }
    let mut weight_by_operator = BTreeMap::new();
    for weight in weights {
        if weight_by_operator
            .insert(weight.operator_cluster_id.clone(), weight)
            .is_some()
        {
            return Err(NoveltyError::DuplicateReviewer);
        }
    }

    let claim_id = &reviews[0].claim_id;
    let mut receipt_ids = BTreeSet::new();
    let mut nodes = BTreeSet::new();
    let mut operators = BTreeSet::new();
    for review in reviews {
        if &review.claim_id != claim_id {
            return Err(NoveltyError::MixedClaims);
        }
        if review.receipt_id.validate().is_err()
            || review.claim_id.validate().is_err()
            || review.reviewer_node_id.validate().is_err()
            || review.operator_cluster_id.validate().is_err()
            || review.corpus_cutoff > review.reviewed_at
            || review.signature.trim().is_empty()
            || review.evidence_root.trim().is_empty()
            || review.corpus_root.trim().is_empty()
        {
            return Err(NoveltyError::InvalidReceipt);
        }
        if !receipt_ids.insert(review.receipt_id.clone())
            || !nodes.insert(review.reviewer_node_id.clone())
            || !operators.insert(review.operator_cluster_id.clone())
        {
            return Err(NoveltyError::DuplicateReviewer);
        }
    }
    if operators.len() < policy.minimum_independent_operators {
        return Err(NoveltyError::InsufficientIndependence);
    }

    let mut novelty_log_odds = logit(policy.prior_probability);
    let mut equivalent_log_odds = logit(1.0 - policy.prior_probability);
    let mut coverage_numerator = 0.0;
    let mut total_weight = 0.0;

    for review in reviews {
        validate_probability(review.material_novelty_probability)?;
        validate_probability(review.known_equivalent_probability)?;
        validate_probability(review.useful_simplification_probability)?;
        validate_probability(review.prior_art_coverage)?;
        validate_probability(review.confidence)?;

        let weight = weight_by_operator
            .get(&review.operator_cluster_id)
            .ok_or_else(|| NoveltyError::MissingWeight(review.operator_cluster_id.to_string()))?;
        if !verifier.verify(review, weight) {
            return Err(NoveltyError::InvalidProof);
        }
        for component in [
            weight.calibration,
            weight.domain_score,
            weight.independence,
            weight.evidence_quality,
        ] {
            validate_probability(component)?;
        }

        let raw_weight = weight.calibration
            * weight.domain_score
            * weight.independence
            * weight.evidence_quality
            * review.confidence;
        let bounded_weight = raw_weight.min(policy.maximum_reviewer_weight).max(0.0);

        novelty_log_odds += bounded_weight * logit(review.material_novelty_probability);
        equivalent_log_odds += bounded_weight * logit(review.known_equivalent_probability);
        coverage_numerator += bounded_weight * review.prior_art_coverage;
        total_weight += bounded_weight;
    }

    if total_weight <= f64::EPSILON {
        return Err(NoveltyError::InvalidValue);
    }

    let posterior_material_novelty = logistic(novelty_log_odds);
    let posterior_known_equivalent = logistic(equivalent_log_odds);
    let weighted_prior_art_coverage = coverage_numerator / total_weight;

    let decision = if weighted_prior_art_coverage < policy.minimum_prior_art_coverage {
        NoveltyDecision::Inconclusive
    } else if posterior_known_equivalent >= policy.known_equivalent_threshold {
        NoveltyDecision::KnownEquivalent
    } else if posterior_material_novelty >= policy.material_novelty_threshold {
        NoveltyDecision::MateriallyNovel
    } else {
        NoveltyDecision::Incremental
    };

    Ok(NoveltyOutcome {
        posterior_material_novelty,
        posterior_known_equivalent,
        weighted_prior_art_coverage,
        decision,
        reviewer_count: reviews.len(),
        independent_operator_count: operators.len(),
    })
}

fn validate_probability(value: f64) -> Result<(), NoveltyError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(NoveltyError::InvalidValue)
    }
}

fn logit(probability: f64) -> f64 {
    let bounded = probability.clamp(1e-9, 1.0 - 1e-9);
    (bounded / (1.0 - bounded)).ln()
}

fn logistic(log_odds: f64) -> f64 {
    1.0 / (1.0 + (-log_odds).exp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use xlemma_core::{ClaimId, NodeId, ReceiptId, TheoryId};

    struct AcceptTestEvidence;

    impl NoveltyEvidenceVerifier for AcceptTestEvidence {
        fn verify(&self, _review: &NoveltyReviewReceipt, _weight: &ReviewerWeight) -> bool {
            true
        }
    }

    struct RejectTestEvidence;

    impl NoveltyEvidenceVerifier for RejectTestEvidence {
        fn verify(&self, _review: &NoveltyReviewReceipt, _weight: &ReviewerWeight) -> bool {
            false
        }
    }

    fn review(label: &str, novelty: f64, equivalent: f64) -> NoveltyReviewReceipt {
        NoveltyReviewReceipt {
            receipt_id: ReceiptId::derive(&format!("receipt-{label}")).unwrap(),
            claim_id: ClaimId::from_canonical_elaborated_type(
                &TheoryId::derive(&"theory").unwrap(),
                "claim",
            )
            .unwrap(),
            reviewer_node_id: NodeId::derive(&format!("node-{label}")).unwrap(),
            operator_cluster_id: OperatorClusterId::derive(&format!("operator-{label}")).unwrap(),
            corpus_root: "blake3:corpus".into(),
            corpus_cutoff: Utc::now(),
            known_equivalent_probability: equivalent,
            material_novelty_probability: novelty,
            useful_simplification_probability: 0.8,
            prior_art_coverage: 0.95,
            confidence: 0.9,
            evidence_root: format!("blake3:evidence-{label}"),
            conflicts_disclosed: Vec::new(),
            reviewed_at: Utc::now(),
            signature: "test".into(),
        }
    }

    fn weight(review: &NoveltyReviewReceipt) -> ReviewerWeight {
        ReviewerWeight {
            operator_cluster_id: review.operator_cluster_id.clone(),
            calibration: 0.9,
            domain_score: 0.9,
            independence: 1.0,
            evidence_quality: 0.9,
        }
    }

    #[test]
    fn independent_high_novelty_reviews_can_clear_threshold() {
        let reviews = vec![review("a", 0.95, 0.02), review("b", 0.90, 0.04)];
        let weights: Vec<_> = reviews.iter().map(weight).collect();
        let outcome = aggregate_novelty(
            &NoveltyPolicy {
                prior_probability: 0.5,
                material_novelty_threshold: 0.75,
                known_equivalent_threshold: 0.75,
                minimum_prior_art_coverage: 0.8,
                maximum_reviewer_weight: 0.75,
                minimum_independent_operators: 2,
            },
            &reviews,
            &weights,
            &AcceptTestEvidence,
        )
        .unwrap();
        assert_eq!(outcome.decision, NoveltyDecision::MateriallyNovel);
    }

    #[test]
    fn operator_independence_is_required() {
        let first = review("a", 0.9, 0.05);
        let mut second = review("b", 0.9, 0.05);
        second.operator_cluster_id = first.operator_cluster_id.clone();
        let reviews = vec![first, second];
        let weights: Vec<_> = reviews.iter().map(weight).collect();
        let result = aggregate_novelty(
            &NoveltyPolicy {
                prior_probability: 0.5,
                material_novelty_threshold: 0.75,
                known_equivalent_threshold: 0.75,
                minimum_prior_art_coverage: 0.8,
                maximum_reviewer_weight: 0.75,
                minimum_independent_operators: 2,
            },
            &reviews,
            &weights,
            &AcceptTestEvidence,
        );
        assert!(matches!(result, Err(NoveltyError::DuplicateReviewer)));
    }

    #[test]
    fn mixed_claim_reviews_are_rejected() {
        let first = review("a", 0.9, 0.05);
        let mut second = review("b", 0.9, 0.05);
        second.claim_id = ClaimId::from_canonical_elaborated_type(
            &TheoryId::derive(&"other-theory").unwrap(),
            "other claim",
        )
        .unwrap();
        let reviews = vec![first, second];
        let weights: Vec<_> = reviews.iter().map(weight).collect();
        assert!(matches!(
            aggregate_novelty(
                &NoveltyPolicy {
                    prior_probability: 0.5,
                    material_novelty_threshold: 0.75,
                    known_equivalent_threshold: 0.75,
                    minimum_prior_art_coverage: 0.8,
                    maximum_reviewer_weight: 0.75,
                    minimum_independent_operators: 2,
                },
                &reviews,
                &weights,
                &AcceptTestEvidence,
            ),
            Err(NoveltyError::MixedClaims)
        ));
    }

    #[test]
    fn unauthenticated_review_or_weight_proof_is_rejected() {
        let reviews = vec![review("a", 0.9, 0.05), review("b", 0.9, 0.05)];
        let weights: Vec<_> = reviews.iter().map(weight).collect();
        assert!(matches!(
            aggregate_novelty(
                &NoveltyPolicy {
                    prior_probability: 0.5,
                    material_novelty_threshold: 0.75,
                    known_equivalent_threshold: 0.75,
                    minimum_prior_art_coverage: 0.8,
                    maximum_reviewer_weight: 0.75,
                    minimum_independent_operators: 2,
                },
                &reviews,
                &weights,
                &RejectTestEvidence,
            ),
            Err(NoveltyError::InvalidProof)
        ));
    }
}
