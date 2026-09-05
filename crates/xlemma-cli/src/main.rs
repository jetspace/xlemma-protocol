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
    evaluate_reproduction, observation_commitment, Amount, AvailabilityReceipt, AxiomProfile,
    CaptureResistanceDashboard, Challenge, ClaimManifest, ComputeReceipt, ConstitutionalCommitment,
    CredentialRevocation, DependencyDividend, EconomicComplianceCertificate, EconomicConstitution,
    EligibleNode, ForkExitPlan, GovernanceProposal, IndependentCredentialAttestation,
    LeanEnvironmentExport, LemmaCapsule, License, NodeCredential, NodeCredentialChain,
    NodeServiceAdvertisement, NodeWorkReceipt, ObjectiveMisconductRecord, ObservationReceipt,
    OperatorCredential, PolicyId, ProofManifest, ProofTrustEvidence, PublicationRecord,
    QuarantineRecord, ReproductionObservation, ResearchComputeCooperative, ResearchCredit,
    ResearchVault, ResearchVerificationCertificate, ResearcherPortabilityManifest,
    ResearcherResidualRight, ResearcherSovereigntyBundle, TheoryId, TrustPolicy,
    TrustPolicyRegistry, TrustPolicyRegistrySnapshot, UserCredential, VerificationJob,
    VerificationProfile,
};
use xlemma_economics::{
    compute_impact_pool_allocation, simulate_discovery, ComputeSavingsEvidence,
    ComputeSavingsPolicy, DiscoverySimulation, FundingReceipt, ImpactPoolAuthorization,
};
use xlemma_storage::{build_bundle_manifest_at, BundleInput};
use xlemma_xlmp::XlmpEnvelope;

#[derive(Parser)]
#[command(name = "xlemma", version, about = "xLemma protocol reference CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Replay synthetic discovery funding, assessment and appeal events; never certifies or pays.
    SimulateDiscovery { scenario: PathBuf },
    /// Derive a domain-separated protocol identifier from a JSON object.
    DeriveId {
        #[arg(value_enum)]
        kind: IdKind,
        input: PathBuf,
    },
    /// Validate a Lean environment export and derive its ClaimID and ProofID.
    LeanExportIds { export: PathBuf, theory: PathBuf },
    /// Evaluate a set of revealed formal observations under a quorum policy.
    EvaluateConsensus {
        policy: PathBuf,
        observations: PathBuf,
    },
    /// Evaluate profile-bound reproduction without voting away divergence.
    EvaluateReproduction {
        profile: PathBuf,
        job: PathBuf,
        observations: PathBuf,
    },
    /// Verify that an export is independently reconstructable without a company database.
    VerifyPortability { manifest: PathBuf },
    /// Verify explicit economic obligations without treating payment as research validity.
    VerifyEconomicCompliance {
        constitution: PathBuf,
        certificate: PathBuf,
    },
    /// Evaluate exact proof/checker evidence against a content-derived trust registry.
    VerifyTrust {
        registry: PathBuf,
        theory: PathBuf,
        proof: PathBuf,
        evidence: PathBuf,
    },
    /// Derive the canonical root of a sorted trust-policy registry snapshot.
    TrustRegistryRoot { registry: PathBuf },
    /// Derive an observation root, commit-reveal binding, and ReceiptID for a draft receipt.
    PrepareReproductionObservation { observation: PathBuf },
    /// Prepare content-derived roots, commitments, and ReceiptIDs for formal observation drafts.
    PrepareFormalObservations { observations: PathBuf },
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
        /// Explicit quote time for deterministic simulation and conformance vectors.
        #[arg(long)]
        quoted_at: Option<DateTime<Utc>>,
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
        /// Explicit RFC 3339 timestamp for byte-for-byte reproducible output.
        #[arg(long)]
        created_at: Option<DateTime<Utc>>,
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
    SovereigntyBundle,
    PortabilityManifest,
    ResidualRight,
    ComputeCooperative,
    CaptureDashboard,
    NodeWorkReceipt,
    ObjectiveMisconduct,
    ConstitutionalCommitment,
    ForkExitPlan,
    GovernanceProposal,
    IssuerAttestation,
    FundingReceipt,
    ReproductionObservation,
    ResearchCertificate,
    EconomicComplianceCertificate,
    AxiomProfile,
    TrustPolicy,
    Challenge,
    Quarantine,
    ComputeReceipt,
    ResearchCredit,
    ResearchVault,
    DependencyDividend,
    License,
    LemmaCapsule,
    Publication,
    AvailabilityReceipt,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::SimulateDiscovery { scenario } => {
            let scenario: DiscoverySimulation = read_json(scenario)?;
            print_json(&simulate_discovery(&scenario)?)?;
        }
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
                IdKind::Artifact => serde_json::from_value::<xlemma_core::ArtifactManifest>(value)?
                    .derive_artifact_id()?
                    .to_string(),
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
                IdKind::SovereigntyBundle => {
                    serde_json::from_value::<ResearcherSovereigntyBundle>(value)?
                        .derive_bundle_id()?
                        .to_string()
                }
                IdKind::PortabilityManifest => {
                    serde_json::from_value::<ResearcherPortabilityManifest>(value)?
                        .derive_manifest_id()?
                        .to_string()
                }
                IdKind::ResidualRight => serde_json::from_value::<ResearcherResidualRight>(value)?
                    .derive_right_id()?
                    .to_string(),
                IdKind::ComputeCooperative => {
                    serde_json::from_value::<ResearchComputeCooperative>(value)?
                        .derive_cooperative_id()?
                        .to_string()
                }
                IdKind::CaptureDashboard => {
                    serde_json::from_value::<CaptureResistanceDashboard>(value)?
                        .derive_dashboard_id()?
                        .to_string()
                }
                IdKind::NodeWorkReceipt => serde_json::from_value::<NodeWorkReceipt>(value)?
                    .derive_receipt_id()?
                    .to_string(),
                IdKind::ObjectiveMisconduct => {
                    serde_json::from_value::<ObjectiveMisconductRecord>(value)?
                        .derive_record_id()?
                        .to_string()
                }
                IdKind::ConstitutionalCommitment => {
                    serde_json::from_value::<ConstitutionalCommitment>(value)?
                        .derive_commitment_id()?
                        .to_string()
                }
                IdKind::ForkExitPlan => serde_json::from_value::<ForkExitPlan>(value)?
                    .derive_plan_id()?
                    .to_string(),
                IdKind::GovernanceProposal => serde_json::from_value::<GovernanceProposal>(value)?
                    .derive_proposal_id()?
                    .to_string(),
                IdKind::IssuerAttestation => {
                    serde_json::from_value::<IndependentCredentialAttestation>(value)?
                        .derive_attestation_id()?
                        .to_string()
                }
                IdKind::FundingReceipt => serde_json::from_value::<FundingReceipt>(value)?
                    .derive_funding_receipt_id()?
                    .to_string(),
                IdKind::ReproductionObservation => {
                    serde_json::from_value::<ReproductionObservation>(value)?
                        .derive_receipt_id()?
                        .to_string()
                }
                IdKind::ResearchCertificate => {
                    serde_json::from_value::<ResearchVerificationCertificate>(value)?
                        .derive_certificate_id()?
                        .to_string()
                }
                IdKind::EconomicComplianceCertificate => {
                    serde_json::from_value::<EconomicComplianceCertificate>(value)?
                        .derive_certificate_id()?
                        .to_string()
                }
                IdKind::AxiomProfile => serde_json::from_value::<AxiomProfile>(value)?
                    .derive_profile_id()?
                    .to_string(),
                IdKind::TrustPolicy => serde_json::from_value::<TrustPolicy>(value)?
                    .derive_policy_id()?
                    .to_string(),
                IdKind::Challenge => serde_json::from_value::<Challenge>(value)?
                    .derive_challenge_id()?
                    .to_string(),
                IdKind::Quarantine => serde_json::from_value::<QuarantineRecord>(value)?
                    .derive_quarantine_id()?
                    .to_string(),
                IdKind::ComputeReceipt => serde_json::from_value::<ComputeReceipt>(value)?
                    .derive_receipt_id()?
                    .to_string(),
                IdKind::ResearchCredit => serde_json::from_value::<ResearchCredit>(value)?
                    .derive_credit_id()?
                    .to_string(),
                IdKind::ResearchVault => serde_json::from_value::<ResearchVault>(value)?
                    .derive_vault_id()?
                    .to_string(),
                IdKind::DependencyDividend => serde_json::from_value::<DependencyDividend>(value)?
                    .derive_dividend_id()?
                    .to_string(),
                IdKind::License => serde_json::from_value::<License>(value)?
                    .derive_license_id()?
                    .to_string(),
                IdKind::LemmaCapsule => serde_json::from_value::<LemmaCapsule>(value)?
                    .derive_lemma_id()?
                    .to_string(),
                IdKind::Publication => serde_json::from_value::<PublicationRecord>(value)?
                    .derive_publication_id()?
                    .to_string(),
                IdKind::AvailabilityReceipt => {
                    serde_json::from_value::<AvailabilityReceipt>(value)?
                        .derive_receipt_id()?
                        .to_string()
                }
            };
            println!("{id}");
        }
        Command::LeanExportIds { export, theory } => {
            let export: LeanEnvironmentExport = read_json(export)?;
            let theory: xlemma_core::TheoryManifest = read_json(theory)?;
            print_json(&export.derive_ids(&theory)?)?;
        }
        Command::EvaluateConsensus {
            policy,
            observations,
        } => {
            let policy: FormalConsensusPolicy = read_json(policy)?;
            let observations: Vec<ObservationReceipt> = read_json(observations)?;
            print_json(&evaluate_formal_consensus(&policy, &observations)?)?;
        }
        Command::EvaluateReproduction {
            profile,
            job,
            observations,
        } => {
            let profile: VerificationProfile = read_json(profile)?;
            let job: VerificationJob = read_json(job)?;
            let observations: Vec<ReproductionObservation> = read_json(observations)?;
            print_json(&evaluate_reproduction(&job, &profile, &observations)?)?;
        }
        Command::VerifyPortability { manifest } => {
            let manifest: ResearcherPortabilityManifest = read_json(manifest)?;
            manifest.validate_reconstructable()?;
            println!("{}", manifest.manifest_id);
        }
        Command::VerifyEconomicCompliance {
            constitution,
            certificate,
        } => {
            let constitution: EconomicConstitution = read_json(constitution)?;
            let certificate: EconomicComplianceCertificate = read_json(certificate)?;
            certificate.validate_against(&constitution)?;
            println!("{}", certificate.certificate_id);
        }
        Command::VerifyTrust {
            registry,
            theory,
            proof,
            evidence,
        } => {
            let snapshot: TrustPolicyRegistrySnapshot = read_json(registry)?;
            let registry = TrustPolicyRegistry::from_snapshot(snapshot)?;
            let theory: xlemma_core::TheoryManifest = read_json(theory)?;
            let proof: ProofManifest = read_json(proof)?;
            let evidence: ProofTrustEvidence = read_json(evidence)?;
            let evaluation = registry.evaluate(&theory, &proof, &evidence)?;
            print_json(&evaluation)?;
            if !evaluation.accepted {
                anyhow::bail!("proof evidence does not satisfy the selected trust policy");
            }
        }
        Command::TrustRegistryRoot { registry } => {
            let snapshot: TrustPolicyRegistrySnapshot = read_json(registry)?;
            println!("{}", snapshot.expected_registry_root()?);
        }
        Command::PrepareReproductionObservation { observation } => {
            let mut observation: ReproductionObservation = read_json(observation)?;
            observation.observation_root = observation.expected_observation_root()?;
            observation.commitment = observation_commitment(
                &observation.job_id,
                observation.verdict,
                &observation.observation_root,
                observation.reveal_salt.as_bytes(),
            );
            observation.receipt_id = observation.derive_receipt_id()?;
            print_json(&observation)?;
        }
        Command::PrepareFormalObservations { observations } => {
            let mut observations: Vec<ObservationReceipt> = read_json(observations)?;
            for observation in &mut observations {
                observation.observation_root = observation.expected_observation_root()?;
                observation.commitment = observation_commitment(
                    &observation.job_id,
                    observation.verdict,
                    &observation.observation_root,
                    observation.reveal_salt.as_bytes(),
                );
                observation.receipt_id = observation.expected_receipt_id()?;
            }
            print_json(&observations)?;
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
            quoted_at,
            risk_premium_bps,
        } => {
            let offers: Vec<ServiceOffer> = read_json(offers)?;
            let work: ExpectedWork = read_json(work)?;
            let estimates: ProtocolSuccessEstimates = read_json(success_estimates)?;
            let quote = quote_quality_adjusted_certification_cost(
                quoted_at.unwrap_or_else(Utc::now),
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
            created_at,
        } => {
            let inputs: Vec<BundleInput> = read_json(inputs)?;
            let bundle = build_bundle_manifest_at(
                &root,
                &inputs,
                lean_toolchain,
                dependency_lock_hash,
                source_commit,
                build_image_digest,
                created_at.unwrap_or_else(Utc::now),
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
