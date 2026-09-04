use xlemma_core::{JobId, ObservationReceipt, ObservationVerdict};

/// Commits a node to its independently produced observation before peer
/// reveals are visible. `observation_root` MUST bind the full execution receipt.
pub fn observation_commitment(
    job_id: &JobId,
    verdict: ObservationVerdict,
    observation_root: &str,
    salt: &[u8],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"xlemma-poir-commit-v1\0");
    update_field(&mut hasher, job_id.as_str().as_bytes());
    update_field(&mut hasher, verdict_label(verdict).as_bytes());
    update_field(&mut hasher, observation_root.as_bytes());
    update_field(&mut hasher, salt);
    format!("blake3:{}", hasher.finalize().to_hex())
}

pub fn verify_reveal(receipt: &ObservationReceipt, salt: &[u8]) -> bool {
    observation_commitment(
        &receipt.job_id,
        receipt.verdict,
        &receipt.observation_root,
        salt,
    ) == receipt.commitment
}

fn update_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn verdict_label(verdict: ObservationVerdict) -> &'static str {
    match verdict {
        ObservationVerdict::Pass => "pass",
        ObservationVerdict::Fail => "fail",
        ObservationVerdict::Error => "error",
        ObservationVerdict::Abstain => "abstain",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use xlemma_core::{JobId, NodeId, OperatorClusterId, ReceiptId};

    #[test]
    fn commitment_binds_verdict_and_root() {
        let job = JobId::derive(&"job").unwrap();
        let salt = "secret";
        let receipt = ObservationReceipt {
            receipt_id: ReceiptId::derive(&"receipt").unwrap(),
            job_id: job.clone(),
            node_id: NodeId::derive(&"node").unwrap(),
            operator_cluster_id: OperatorClusterId::derive(&"operator").unwrap(),
            checker_family: None,
            checker_name: "reference-checker".into(),
            checker_version: "0.1.0".into(),
            checker_binary_digest: "sha256:checker".into(),
            infrastructure_provider: "provider-a".into(),
            region: "region-a".into(),
            artifact_root: "artifact".into(),
            environment_root: "environment".into(),
            dependency_root: "dependencies".into(),
            axiom_set_root: "axioms".into(),
            execution_trace_root: "trace".into(),
            observation_root: "observation".into(),
            verdict: ObservationVerdict::Pass,
            commitment: observation_commitment(
                &job,
                ObservationVerdict::Pass,
                "observation",
                salt.as_bytes(),
            ),
            reveal_salt: salt.into(),
            committed_at: Utc::now(),
            revealed_at: Utc::now(),
            signature: "test".into(),
        };
        assert!(verify_reveal(&receipt, salt.as_bytes()));
        assert!(!verify_reveal(&receipt, b"wrong"));
    }
}
