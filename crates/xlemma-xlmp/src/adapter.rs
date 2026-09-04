//! Provider-neutral boundaries implemented by concrete xLemma integrations.

use crate::XlmpEnvelope;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use xlemma_core::{
    Amount, ArtifactId, ArtifactManifest, AvailabilityReceipt, ClaimId, ComputeQuoteId,
    ComputeReceipt, JobId, ObservationReceipt, PaymentReceipt, PolicyId, ProofId, ReceiptId,
    ReproductionObservation, TheoryId, TransportReceipt, VerificationJob, VerificationProfile,
};

#[derive(Clone, Debug, PartialEq, Eq, Error, Serialize, Deserialize)]
#[error("adapter {adapter} failed: {reason}")]
pub struct AdapterError {
    pub adapter: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProverRequest {
    pub job_id: JobId,
    pub theory_id: TheoryId,
    pub claim_id: ClaimId,
    pub input_artifact_id: ArtifactId,
    pub verification_policy_id: PolicyId,
    pub context_root: String,
    pub instruction_root: String,
    /// Adapter input supplied out-of-band from the canonical public message.
    /// Implementations MUST NOT place confidential values in logs or receipts.
    pub parameters: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProverArtifact {
    pub artifact_id: ArtifactId,
    pub proof_id: Option<ProofId>,
    pub media_type: String,
    pub artifact_root: String,
    pub compute_receipt: ComputeReceipt,
}

/// ASTRA may be a premier implementation of this interface, but no ASTRA
/// response can satisfy `VerifierAdapter` or self-certify a candidate.
#[async_trait]
pub trait ResearchProver: Send + Sync {
    async fn formalize(&self, request: ProverRequest) -> Result<ProverArtifact, AdapterError>;
    async fn propose(&self, request: ProverRequest) -> Result<ProverArtifact, AdapterError>;
    async fn prove(&self, request: ProverRequest) -> Result<ProverArtifact, AdapterError>;
    async fn repair(&self, request: ProverRequest) -> Result<ProverArtifact, AdapterError>;
    async fn explain(&self, request: ProverRequest) -> Result<ProverArtifact, AdapterError>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierRequest {
    pub job: VerificationJob,
    pub proof_id: ProofId,
    pub exact_challenge_hash: String,
    pub dependency_root: String,
    pub axiom_policy_id: PolicyId,
}

/// Lean is the default XLMP/1 verifier backend. Other formal systems may
/// implement this interface without changing protocol identities or messages.
#[async_trait]
pub trait VerifierAdapter: Send + Sync {
    fn family(&self) -> &str;
    async fn reproduce(&self, request: VerifierRequest)
        -> Result<ObservationReceipt, AdapterError>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproductionRequest {
    pub job: VerificationJob,
    pub profile: VerificationProfile,
    pub exact_input_root: String,
}

/// Computational, statistical, simulation, empirical, and hybrid backends use
/// this provider-neutral boundary. The returned receipt remains untrusted
/// until its content identity, signature, profile evidence, and operator
/// independence are checked.
#[async_trait]
pub trait ReproductionAdapter: Send + Sync {
    fn implementation(&self) -> &str;
    async fn reproduce(
        &self,
        request: ReproductionRequest,
    ) -> Result<ReproductionObservation, AdapterError>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentInstruction {
    pub job_id: JobId,
    pub quote_id: ComputeQuoteId,
    pub payer: String,
    pub payee: String,
    pub maximum_authorization: Amount,
    pub payment_terms_root: String,
    pub valid_until: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentAuthorization {
    pub authorization_id: String,
    pub job_id: JobId,
    pub adapter: String,
    pub authorized: Amount,
    pub authorization_reference: String,
    pub expires_at: DateTime<Utc>,
    pub signature: String,
}

/// x402, stablecoin rails, native-chain payments, backed research credits,
/// grants, bounty escrow, and invoicing are peer implementations of this API.
#[async_trait]
pub trait PaymentAdapter: Send + Sync {
    async fn authorize(
        &self,
        instruction: PaymentInstruction,
    ) -> Result<PaymentAuthorization, AdapterError>;

    async fn settle(
        &self,
        authorization: PaymentAuthorization,
        actual_charge: Amount,
    ) -> Result<PaymentReceipt, AdapterError>;
}

#[async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn put(
        &self,
        artifact_id: ArtifactId,
        manifest: ArtifactManifest,
    ) -> Result<AvailabilityReceipt, AdapterError>;

    async fn get(&self, artifact_id: ArtifactId) -> Result<Vec<u8>, AdapterError>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalityReceipt {
    pub receipt_id: ReceiptId,
    pub adapter: String,
    pub network: String,
    pub state_root: String,
    pub finalized_at: DateTime<Utc>,
    pub finality_reference: String,
    pub signature: String,
}

/// Chains provide ordering or economic finality; they do not determine proof
/// validity, attribution, novelty, or rights.
#[async_trait]
pub trait FinalityAdapter: Send + Sync {
    async fn anchor_state(
        &self,
        state_root: String,
        metadata: BTreeMap<String, String>,
    ) -> Result<FinalityReceipt, AdapterError>;
}

/// HTTP, libp2p, WebSocket, x402, and chain event streams may all carry the
/// same canonical envelope without changing its meaning.
#[async_trait]
pub trait TransportAdapter: Send + Sync {
    async fn send(&self, envelope: &XlmpEnvelope) -> Result<TransportReceipt, AdapterError>;
}
