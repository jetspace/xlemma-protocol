use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::PathBuf};
use xlemma_compute_curve::{quote_verified_proof_cost, ExpectedWork, ServiceOffer};
use xlemma_consensus::{
    eligible_set_root, evaluate_formal_consensus, randomness_commitment, select_committee,
    FormalConsensusPolicy,
};
use xlemma_core::{
    Amount, ArtifactId, ClaimManifest, CommitteeSortitionRequest, EligibleNode,
    NodeServiceAdvertisement, ObservationReceipt, ProofManifest, TheoryId,
};
use xlemma_economics::{compute_savings_dividend, ComputeSavingsEvidence, ComputeSavingsPolicy};
use xlemma_storage::{build_bundle_manifest, BundleInput};

#[derive(Parser)]
#[command(name = "xlemma", version, about = "xLemma protocol reference CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Derive a domain-separated protocol identifier from a JSON object.
    DeriveId {
        #[arg(value_enum)]
        kind: IdKind,
        input: PathBuf,
    },
    /// Evaluate a set of revealed formal observations under a quorum policy.
    EvaluateConsensus {
        policy: PathBuf,
        observations: PathBuf,
    },
    /// Commit a public randomness reveal for a future sortition request.
    CommitteeRandomness {
        #[arg(long)]
        revealed_seed: String,
    },
    /// Derive the canonical root of an exact eligible-node JSON array.
    EligibleSetRoot { eligible_nodes: PathBuf },
    /// Reproduce an auditable committee selection from committed inputs.
    SelectCommittee {
        request: PathBuf,
        eligible_nodes: PathBuf,
        #[arg(long)]
        revealed_seed: String,
        #[arg(long)]
        selected_at: DateTime<Utc>,
    },
    /// Quote a verified proof from service offers and expected work.
    Quote {
        offers: PathBuf,
        work: PathBuf,
        #[arg(long)]
        deadline: DateTime<Utc>,
        #[arg(long)]
        gold_probability: f64,
        #[arg(long)]
        novelty_probability: f64,
        #[arg(long, default_value_t = 500)]
        risk_premium_bps: u16,
    },
    /// Calculate a conservative, revenue-capped compute-savings dividend.
    ComputeDividend {
        evidence: PathBuf,
        policy: PathBuf,
        downstream_net_revenue: PathBuf,
    },
    /// Build a content-addressed artifact manifest from explicit files.
    Pack {
        root: PathBuf,
        inputs: PathBuf,
        #[arg(long)]
        lean_toolchain: String,
        #[arg(long)]
        dependency_lock_hash: String,
        #[arg(long)]
        source_commit: Option<String>,
        #[arg(long)]
        build_image_digest: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum IdKind {
    Theory,
    Claim,
    Proof,
    Artifact,
    Advertisement,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::DeriveId { kind, input } => {
            let value: serde_json::Value = read_json(input)?;
            let id = match kind {
                IdKind::Theory => TheoryId::derive(&value)?.to_string(),
                IdKind::Claim => serde_json::from_value::<ClaimManifest>(value)?
                    .derive_claim_id()?
                    .to_string(),
                IdKind::Proof => serde_json::from_value::<ProofManifest>(value)?
                    .derive_proof_id()?
                    .to_string(),
                IdKind::Artifact => ArtifactId::derive(&value)?.to_string(),
                IdKind::Advertisement => serde_json::from_value::<NodeServiceAdvertisement>(value)?
                    .derive_advertisement_id()?
                    .to_string(),
            };
            println!("{id}");
        }
        Command::EvaluateConsensus {
            policy,
            observations,
        } => {
            let policy: FormalConsensusPolicy = read_json(policy)?;
            let observations: Vec<ObservationReceipt> = read_json(observations)?;
            print_json(&evaluate_formal_consensus(&policy, &observations)?)?;
        }
        Command::CommitteeRandomness { revealed_seed } => {
            println!("{}", randomness_commitment(revealed_seed.as_bytes()));
        }
        Command::EligibleSetRoot { eligible_nodes } => {
            let nodes: Vec<EligibleNode> = read_json(eligible_nodes)?;
            println!("{}", eligible_set_root(&nodes)?);
        }
        Command::SelectCommittee {
            request,
            eligible_nodes,
            revealed_seed,
            selected_at,
        } => {
            let request: CommitteeSortitionRequest = read_json(request)?;
            let nodes: Vec<EligibleNode> = read_json(eligible_nodes)?;
            print_json(&select_committee(
                &request,
                revealed_seed.as_bytes(),
                &nodes,
                selected_at,
            )?)?;
        }
        Command::Quote {
            offers,
            work,
            deadline,
            gold_probability,
            novelty_probability,
            risk_premium_bps,
        } => {
            let offers: Vec<ServiceOffer> = read_json(offers)?;
            let work: ExpectedWork = read_json(work)?;
            let quote = quote_verified_proof_cost(
                Utc::now(),
                deadline,
                &work,
                &offers,
                gold_probability,
                novelty_probability,
                risk_premium_bps,
            )?;
            print_json(&quote)?;
        }
        Command::ComputeDividend {
            evidence,
            policy,
            downstream_net_revenue,
        } => {
            let evidence: ComputeSavingsEvidence = read_json(evidence)?;
            let policy: ComputeSavingsPolicy = read_json(policy)?;
            let revenue: Amount = read_json(downstream_net_revenue)?;
            print_json(&compute_savings_dividend(&evidence, &policy, &revenue)?)?;
        }
        Command::Pack {
            root,
            inputs,
            lean_toolchain,
            dependency_lock_hash,
            source_commit,
            build_image_digest,
        } => {
            let inputs: Vec<BundleInput> = read_json(inputs)?;
            let bundle = build_bundle_manifest(
                &root,
                &inputs,
                lean_toolchain,
                dependency_lock_hash,
                source_commit,
                build_image_digest,
            )?;
            print_json(&bundle)?;
        }
    }
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: PathBuf) -> Result<T> {
    let bytes = fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse JSON from {}", path.display()))
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
