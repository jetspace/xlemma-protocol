//! Lean build and checker execution boundaries.
//!
//! `LocalCommandRunner` is a development utility. Production nodes MUST run
//! hostile proof artifacts in a hardened, no-network sandbox and export proof
//! objects for replay outside that sandbox.

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf, process::Stdio};
use thiserror::Error;
use tokio::{io::AsyncReadExt, process::Command, time::timeout};
use xlemma_core::{
    ArtifactId, CheckerExecution, CheckerFamily, ClaimId, JobId, LeanVerificationReceipt,
    NodeId, ObservationVerdict, OperatorClusterId, PolicyId, ProofId, ReceiptId, TheoryId,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub image_digest: String,
    pub network_disabled: bool,
    pub read_only_root: bool,
    pub cpu_limit_millis: u64,
    pub memory_limit_bytes: u64,
    pub process_limit: u32,
    pub timeout_seconds: u64,
    pub writable_paths: Vec<String>,
    pub seccomp_profile_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeanVerificationRequest {
    pub job_id: JobId,
    pub claim_id: ClaimId,
    pub proof_id: ProofId,
    pub theory_id: TheoryId,
    pub artifact_id: ArtifactId,
    pub workspace: PathBuf,
    pub trusted_challenge_path: PathBuf,
    pub proof_project_path: PathBuf,
    pub lean_toolchain: String,
    pub dependency_root: String,
    pub axiom_policy_id: PolicyId,
    pub permitted_axioms: Vec<String>,
    pub sandbox_policy: SandboxPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandSpec {
    pub program: String,
    pub arguments: Vec<String>,
    pub working_directory: PathBuf,
    pub environment: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub trace_root: String,
}

#[derive(Debug, Error)]
pub enum LeanVerificationError {
    #[error("unsafe sandbox policy: production verification requires no network and a read-only root")]
    UnsafeSandboxPolicy,
    #[error("command I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("command exceeded sandbox timeout")]
    Timeout,
    #[error("build failed: {0}")]
    BuildFailed(String),
    #[error("trusted challenge did not match")]
    ChallengeMismatch,
    #[error("observed an unpermitted axiom: {0}")]
    UnpermittedAxiom(String),
    #[error("checker family diverged")]
    CheckerDivergence,
}

#[async_trait]
pub trait SandboxRunner: Send + Sync {
    async fn run(
        &self,
        policy: &SandboxPolicy,
        command: &CommandSpec,
    ) -> Result<CommandResult, LeanVerificationError>;
}

/// Development-only command runner. It enforces a wall-clock timeout but does
/// not provide process, filesystem, seccomp, container, or network isolation.
pub struct LocalCommandRunner;

#[async_trait]
impl SandboxRunner for LocalCommandRunner {
    async fn run(
        &self,
        policy: &SandboxPolicy,
        command: &CommandSpec,
    ) -> Result<CommandResult, LeanVerificationError> {
        let mut child = Command::new(&command.program)
            .args(&command.arguments)
            .current_dir(&command.working_directory)
            .env_clear()
            .envs(&command.environment)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let mut stdout = child.stdout.take().expect("stdout was piped");
        let mut stderr = child.stderr.take().expect("stderr was piped");
        let mut stdout_bytes = Vec::new();
        let mut stderr_bytes = Vec::new();
        let wait = async {
            let (stdout_result, stderr_result, status_result) = tokio::join!(
                stdout.read_to_end(&mut stdout_bytes),
                stderr.read_to_end(&mut stderr_bytes),
                child.wait(),
            );
            stdout_result?;
            stderr_result?;
            let status = status_result?;
            Ok::<_, std::io::Error>(status)
        };

        let status = match timeout(
            std::time::Duration::from_secs(policy.timeout_seconds),
            wait,
        )
        .await
        {
            Ok(result) => result?,
            Err(_) => return Err(LeanVerificationError::Timeout),
        };

        let mut trace_hasher = blake3::Hasher::new();
        trace_hasher.update(b"xlemma-command-trace-v1\0");
        trace_hasher.update(command.program.as_bytes());
        for argument in &command.arguments {
            trace_hasher.update(b"\0");
            trace_hasher.update(argument.as_bytes());
        }
        trace_hasher.update(&stdout_bytes);
        trace_hasher.update(&stderr_bytes);

        Ok(CommandResult {
            exit_code: status.code(),
            stdout: String::from_utf8_lossy(&stdout_bytes).into_owned(),
            stderr: String::from_utf8_lossy(&stderr_bytes).into_owned(),
            timed_out: false,
            trace_root: format!("blake3:{}", trace_hasher.finalize().to_hex()),
        })
    }
}

#[derive(Clone, Debug)]
pub struct CheckerNodeIdentity {
    pub node_id: NodeId,
    pub operator_cluster_id: OperatorClusterId,
    pub infrastructure_provider: String,
    pub region: String,
}

pub struct LeanVerifier<R> {
    runner: R,
    official_kernel: CheckerNodeIdentity,
    independent_checker: CheckerNodeIdentity,
}

impl<R: SandboxRunner> LeanVerifier<R> {
    pub fn new(
        runner: R,
        official_kernel: CheckerNodeIdentity,
        independent_checker: CheckerNodeIdentity,
    ) -> Self {
        Self {
            runner,
            official_kernel,
            independent_checker,
        }
    }

    pub async fn verify(
        &self,
        request: &LeanVerificationRequest,
    ) -> Result<LeanVerificationReceipt, LeanVerificationError> {
        if !request.sandbox_policy.network_disabled || !request.sandbox_policy.read_only_root {
            return Err(LeanVerificationError::UnsafeSandboxPolicy);
        }

        let base_env = BTreeMap::from([
            ("HOME".to_owned(), "/tmp/xlemma-home".to_owned()),
            ("PATH".to_owned(), "/usr/local/bin:/usr/bin:/bin".to_owned()),
            ("NO_COLOR".to_owned(), "1".to_owned()),
        ]);

        let build = self
            .runner
            .run(
                &request.sandbox_policy,
                &CommandSpec {
                    program: "lake".to_owned(),
                    arguments: vec!["build".to_owned()],
                    working_directory: request.proof_project_path.clone(),
                    environment: base_env.clone(),
                },
            )
            .await?;
        if build.exit_code != Some(0) {
            return Err(LeanVerificationError::BuildFailed(build.stderr));
        }

        let kernel = self
            .runner
            .run(
                &request.sandbox_policy,
                &CommandSpec {
                    program: "lake".to_owned(),
                    arguments: vec![
                        "env".to_owned(),
                        "lean4checker".to_owned(),
                        "--fresh".to_owned(),
                    ],
                    working_directory: request.proof_project_path.clone(),
                    environment: base_env.clone(),
                },
            )
            .await?;

        let comparator = self
            .runner
            .run(
                &request.sandbox_policy,
                &CommandSpec {
                    program: "comparator".to_owned(),
                    arguments: vec![
                        "--challenge".to_owned(),
                        request.trusted_challenge_path.display().to_string(),
                        "--proof".to_owned(),
                        request.proof_project_path.display().to_string(),
                        "--checker".to_owned(),
                        "nanoda".to_owned(),
                    ],
                    working_directory: request.workspace.clone(),
                    environment: base_env,
                },
            )
            .await?;

        let kernel_verdict = verdict_from_exit(kernel.exit_code);
        let independent_verdict = verdict_from_exit(comparator.exit_code);
        if kernel_verdict != independent_verdict {
            return Err(LeanVerificationError::CheckerDivergence);
        }

        let exact_challenge_matched = comparator.stdout.contains("challenge matched")
            || comparator.stdout.contains("SUCCESS");
        if independent_verdict == ObservationVerdict::Pass && !exact_challenge_matched {
            return Err(LeanVerificationError::ChallengeMismatch);
        }

        // The production exporter supplies a machine-readable axiom report.
        // This reference implementation recognizes `AXIOM:` lines.
        let observed_axioms = extract_axioms(&format!(
            "{}\n{}\n{}",
            build.stdout, kernel.stdout, comparator.stdout
        ));
        for axiom in &observed_axioms {
            if !request.permitted_axioms.contains(axiom) {
                return Err(LeanVerificationError::UnpermittedAxiom(axiom.clone()));
            }
        }

        let checker_executions = vec![
            CheckerExecution {
                checker_family: CheckerFamily::LeanKernel,
                checker_name: "lean4checker".to_owned(),
                checker_version: request.lean_toolchain.clone(),
                binary_digest: "CONFIGURE_AT_DEPLOYMENT".to_owned(),
                node_id: self.official_kernel.node_id.clone(),
                operator_cluster_id: self.official_kernel.operator_cluster_id.clone(),
                infrastructure_provider: Some(
                    self.official_kernel.infrastructure_provider.clone(),
                ),
                region: Some(self.official_kernel.region.clone()),
                verdict: kernel_verdict,
                execution_trace_root: kernel.trace_root,
            },
            CheckerExecution {
                checker_family: CheckerFamily::Nanoda,
                checker_name: "nanoda-via-comparator".to_owned(),
                checker_version: "CONFIGURE_AT_DEPLOYMENT".to_owned(),
                binary_digest: "CONFIGURE_AT_DEPLOYMENT".to_owned(),
                node_id: self.independent_checker.node_id.clone(),
                operator_cluster_id: self.independent_checker.operator_cluster_id.clone(),
                infrastructure_provider: Some(
                    self.independent_checker.infrastructure_provider.clone(),
                ),
                region: Some(self.independent_checker.region.clone()),
                verdict: independent_verdict,
                execution_trace_root: comparator.trace_root,
            },
        ];

        let receipt_material = serde_json::json!({
            "job_id": request.job_id,
            "claim_id": request.claim_id,
            "proof_id": request.proof_id,
            "artifact_id": request.artifact_id,
            "checkers": checker_executions,
            "axioms": observed_axioms,
        });

        Ok(LeanVerificationReceipt {
            receipt_id: ReceiptId::derive(&receipt_material)
                .expect("verification receipt is serializable"),
            job_id: request.job_id.clone(),
            claim_id: request.claim_id.clone(),
            proof_id: request.proof_id.clone(),
            theory_id: request.theory_id.clone(),
            artifact_id: request.artifact_id.clone(),
            exact_challenge_matched,
            lean_toolchain: request.lean_toolchain.clone(),
            dependency_root: request.dependency_root.clone(),
            axiom_policy_id: request.axiom_policy_id.clone(),
            observed_axioms,
            sandbox_image_digest: request.sandbox_policy.image_digest.clone(),
            checker_executions,
            verdict: kernel_verdict,
            verified_at: Utc::now(),
            aggregate_signature: "UNSIGNED_REFERENCE_RECEIPT".to_owned(),
        })
    }
}

fn verdict_from_exit(code: Option<i32>) -> ObservationVerdict {
    match code {
        Some(0) => ObservationVerdict::Pass,
        Some(_) => ObservationVerdict::Fail,
        None => ObservationVerdict::Error,
    }
}

fn extract_axioms(output: &str) -> Vec<String> {
    let mut axioms: Vec<_> = output
        .lines()
        .filter_map(|line| line.trim().strip_prefix("AXIOM:"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect();
    axioms.sort();
    axioms.dedup();
    axioms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axiom_inventory_is_sorted_and_deduplicated() {
        let axioms = extract_axioms(
            "noise\nAXIOM: Classical.choice\nAXIOM: propext\nAXIOM: Classical.choice\n",
        );
        assert_eq!(
            axioms,
            vec!["Classical.choice".to_owned(), "propext".to_owned()]
        );
    }

    #[test]
    fn nonzero_exit_is_a_failure_not_an_abstention() {
        assert_eq!(verdict_from_exit(Some(0)), ObservationVerdict::Pass);
        assert_eq!(verdict_from_exit(Some(1)), ObservationVerdict::Fail);
        assert_eq!(verdict_from_exit(None), ObservationVerdict::Error);
    }
}
