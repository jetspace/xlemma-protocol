//! x402 V2 transport objects and the xLemma extension.
//!
//! This crate does not reimplement chain-specific settlement. Use an audited
//! official or production x402 SDK/facilitator behind `PaymentFacilitator`.

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Utc};
use http::{HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;
use xlemma_core::{
    Amount, ClaimId, ComputeQuoteId, JobId, PaymentReceipt, PolicyId, ProofId, ResearcherId,
};

pub const PAYMENT_REQUIRED_HEADER: &str = "payment-required";
pub const PAYMENT_SIGNATURE_HEADER: &str = "payment-signature";
pub const PAYMENT_RESPONSE_HEADER: &str = "payment-response";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentScheme {
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "upto")]
    Upto,
    #[serde(rename = "batch-settlement")]
    BatchSettlement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceDescription {
    pub url: String,
    pub description: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentRequirement {
    pub scheme: PaymentScheme,
    pub network: String,
    pub amount: String,
    pub asset: String,
    #[serde(rename = "payTo")]
    pub pay_to: String,
    #[serde(rename = "maxTimeoutSeconds")]
    pub max_timeout_seconds: u64,
    #[serde(default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XLemmaPaymentExtension {
    pub protocol: String,
    #[serde(rename = "jobId")]
    pub job_id: JobId,
    #[serde(rename = "researcherId")]
    pub researcher_id: ResearcherId,
    #[serde(rename = "claimId")]
    pub claim_id: ClaimId,
    #[serde(rename = "proofId", skip_serializing_if = "Option::is_none")]
    pub proof_id: Option<ProofId>,
    #[serde(rename = "artifactCommitment")]
    pub artifact_commitment: String,
    #[serde(rename = "computeQuoteId")]
    pub compute_quote_id: ComputeQuoteId,
    #[serde(rename = "requiredVerificationPolicy")]
    pub required_verification_policy: PolicyId,
    #[serde(rename = "modelPolicy")]
    pub model_policy: String,
    #[serde(rename = "rightsManifestHash")]
    pub rights_manifest_hash: String,
    #[serde(rename = "revenueRouteHash")]
    pub revenue_route_hash: String,
    #[serde(rename = "deliveryMode")]
    pub delivery_mode: String,
    #[serde(rename = "validUntil")]
    pub valid_until: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentRequired {
    #[serde(rename = "x402Version")]
    pub x402_version: u8,
    pub error: String,
    pub resource: ResourceDescription,
    pub accepts: Vec<PaymentRequirement>,
    #[serde(default)]
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentSignatureEnvelope {
    #[serde(rename = "x402Version")]
    pub x402_version: u8,
    pub scheme: PaymentScheme,
    pub network: String,
    pub payload: Value,
    #[serde(rename = "paymentIdentifier")]
    pub payment_identifier: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentResponseEnvelope {
    pub success: bool,
    pub transaction: Option<String>,
    pub network: String,
    pub payer: Option<String>,
    pub settled_amount: Option<String>,
    pub error_reason: Option<String>,
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResearchServiceKind {
    FixedLeanCheck,
    VariableAstraSearch,
    MeteredRepair,
    RepeatedProofState,
    CertifiedBundleDownload,
    ContinuousResearchSession,
}

pub fn recommended_scheme(service: ResearchServiceKind) -> PaymentScheme {
    match service {
        ResearchServiceKind::FixedLeanCheck
        | ResearchServiceKind::CertifiedBundleDownload => PaymentScheme::Exact,
        ResearchServiceKind::VariableAstraSearch | ResearchServiceKind::MeteredRepair => {
            PaymentScheme::Upto
        }
        ResearchServiceKind::RepeatedProofState
        | ResearchServiceKind::ContinuousResearchSession => PaymentScheme::BatchSettlement,
    }
}

#[derive(Debug, Error)]
pub enum X402Error {
    #[error("invalid base64 header: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("invalid JSON header: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid HTTP header value: {0}")]
    Header(#[from] http::header::InvalidHeaderValue),
    #[error("required payment header is missing")]
    MissingHeader,
    #[error("payment verification failed: {0}")]
    Verification(String),
    #[error("payment settlement failed: {0}")]
    Settlement(String),
}

pub fn encode_header<T: Serialize>(value: &T) -> Result<HeaderValue, X402Error> {
    let bytes = serde_json::to_vec(value)?;
    let encoded = STANDARD.encode(bytes);
    Ok(HeaderValue::from_bytes(encoded.as_bytes())?)
}

pub fn decode_header<T: DeserializeOwned>(value: &HeaderValue) -> Result<T, X402Error> {
    let decoded = STANDARD.decode(value.as_bytes())?;
    Ok(serde_json::from_slice(&decoded)?)
}

pub fn insert_payment_required(
    headers: &mut HeaderMap,
    required: &PaymentRequired,
) -> Result<(), X402Error> {
    let _ = headers.insert(PAYMENT_REQUIRED_HEADER, encode_header(required)?);
    Ok(())
}

pub fn read_payment_signature(
    headers: &HeaderMap,
) -> Result<PaymentSignatureEnvelope, X402Error> {
    let value = headers
        .get(PAYMENT_SIGNATURE_HEADER)
        .ok_or(X402Error::MissingHeader)?;
    decode_header(value)
}

#[derive(Clone, Debug)]
pub struct SettlementRequest {
    pub job_id: JobId,
    pub maximum_authorization: Amount,
    pub actual_charge: Amount,
    pub signature: PaymentSignatureEnvelope,
    pub pay_to: String,
}

#[async_trait]
pub trait PaymentFacilitator: Send + Sync {
    async fn verify_authorization(
        &self,
        requirement: &PaymentRequirement,
        signature: &PaymentSignatureEnvelope,
    ) -> Result<(), X402Error>;

    async fn settle(&self, request: SettlementRequest) -> Result<PaymentReceipt, X402Error>;
}

/// Adds the xLemma extension without conflating payment validity with proof
/// validity. The proof certificate is delivered as a separate resource or
/// extension only after independent verification.
pub fn with_xlemma_extension(
    mut required: PaymentRequired,
    extension: &XLemmaPaymentExtension,
) -> Result<PaymentRequired, X402Error> {
    let _ = required
        .extensions
        .insert("xlemma".to_owned(), serde_json::to_value(extension)?);
    Ok(required)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn headers_round_trip() {
        let envelope = PaymentSignatureEnvelope {
            x402_version: 2,
            scheme: PaymentScheme::Upto,
            network: "eip155:8453".to_owned(),
            payload: json!({"authorization": "0xabc"}),
            payment_identifier: "job-1-attempt-1".to_owned(),
        };
        let encoded = encode_header(&envelope).unwrap();
        let decoded: PaymentSignatureEnvelope = decode_header(&encoded).unwrap();
        assert_eq!(decoded.payment_identifier, envelope.payment_identifier);
    }
}
