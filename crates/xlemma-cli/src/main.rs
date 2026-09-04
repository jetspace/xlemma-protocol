use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::PathBuf};
use xlemma_compute_curve::{
    quote_quality_adjusted_certification_cost, ExpectedWork, ProtocolSuccessEstimates, ServiceOffer,
};
use xlemma_consensus::{
    eligible_set_root, evaluate_formal_consensus, randomness_commitment, FormalConsensusPolicy,
};
use xlemma_core::{
    Amount, ArtifactId, ClaimManifest, CredentialRevocation, EligibleNode, NodeCredential,
    NodeCredentialChain, NodeServiceAdvertisement, ObservationReceipt, OperatorCredential,
    PolicyId, ProofManifest, TheoryId, UserCredential,
};
use xlemma_economics::{
    compute_impact_pool_allocation, ComputeSavingsEvidence, ComputeSavingsPolicy,
    ImpactPoolAuthorization,
};
use xlemma_storage::{build_bundle_manifest, BundleInput};
use xlemma_xlmp::XlmpEnvelope;

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
    /// Derive the canonical root of a participant/operator/node credential chain.
    CredentialChainRoot { credential_chain: PathBuf },
    /// Derive the canonical identifier of an XLMP envelope's message content.
    MessageId { envelope: PathBuf },
    /// Quote a verified proof from service offers and expected work.
    Quote {
        offers: PathBuf,
        work: PathBuf,
        success_estimates: PathBuf,
        /// Estimator key authorized by the deployment for this quote.
        #[arg(long)]
        trusted_estimator: String,
        #[arg(long)]
        deadline: DateTime<Utc>,
        #[arg(long, default_value_t = 500)]
        risk_premium_bps: u16,
    },
    /// Allocate a conservative compute-savings signal from an authorized impact pool.
    ComputeImpact {
        evidence: PathBuf,
        policy: PathBuf,
        downstream_net_revenue: PathBuf,
        impact_pool_authorization: PathBuf,
        /// Impact-pool authorizer key trusted by the deployment.
        #[arg(long)]
        trusted_authorizer: String,
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
    UserCredential,
    OperatorCredential,
    NodeCredential,
    CredentialRevocation,
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
                IdKind::UserCredential => serde_json::from_value::<UserCredential>(value)?
                    .derive_credential_id()?
                    .to_string(),
                IdKind::OperatorCredential => serde_json::from_value::<OperatorCredential>(value)?
                    .derive_credential_id()?
                    .to_string(),
                IdKind::NodeCredential => serde_json::from_value::<NodeCredential>(value)?
                    .derive_credential_id()?
                    .to_string(),
                IdKind::CredentialRevocation => {
                    serde_json::from_value::<CredentialRevocation>(value)?
                        .derive_revocation_id()?
                        .to_string()
                }
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
        Command::CredentialChainRoot { credential_chain } => {
            let chain: NodeCredentialChain = read_json(credential_chain)?;
            println!("{}", chain.derive_chain_root()?);
        }
        Command::MessageId { envelope } => {
            let envelope: XlmpEnvelope = read_json(envelope)?;
            println!("{}", envelope.expected_message_id()?);
        }
        Command::Quote {
            offers,
            work,
            success_estimates,
            trusted_estimator,
            deadline,
            risk_premium_bps,
        } => {
            let offers: Vec<ServiceOffer> = read_json(offers)?;
            let work: ExpectedWork = read_json(work)?;
            let estimates: ProtocolSuccessEstimates = read_json(success_estimates)?;
            let quote = quote_quality_adjusted_certification_cost(
                Utc::now(),
                deadline,
                &work,
                &offers,
                &estimates,
                &|signer: &str, _: &PolicyId| signer == trusted_estimator,
                risk_premium_bps,
            )?;
            print_json(&quote)?;
        }
        Command::ComputeImpact {
            evidence,
            policy,
            downstream_net_revenue,
            impact_pool_authorization,
            trusted_authorizer,
        } => {
            let evidence: ComputeSavingsEvidence = read_json(evidence)?;
            let policy: ComputeSavingsPolicy = read_json(policy)?;
            let revenue: Amount = read_json(downstream_net_revenue)?;
            let authorization: ImpactPoolAuthorization = read_json(impact_pool_authorization)?;
            print_json(&compute_impact_pool_allocation(
                &evidence,
                &policy,
                &revenue,
                &authorization,
                &|authorizer: &str, _: &str| authorizer == trusted_authorizer,
            )?)?;
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
