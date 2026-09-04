//! x402 V2 payment-adapter objects for XLMP/1.
//!
//! x402 transports payment authorization; it does not define xLemma research
//! state, verification, rights, or consensus. This crate does not reimplement
//! chain-specific settlement. Use an audited SDK/facilitator behind
//! `PaymentFacilitator`.

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Utc};
use http::{HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;
use xlemma_core::{
    Amount, ClaimId, ComputeQuoteId, JobId, MessageId, PaymentReceipt, PolicyId, ProofId,
    ResearcherId, XLMP_VERSION,
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
pub struct XlmpPaymentExtension {
    pub protocol: String,
    #[serde(rename = "xlmpMessageId")]
    pub xlmp_message_id: MessageId,
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
        ResearchServiceKind::FixedLeanCheck | ResearchServiceKind::CertifiedBundleDownload => {
            PaymentScheme::Exact
        }
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
    #[error("invalid XLMP payment extension: {0}")]
    InvalidExtension(String),
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

pub fn read_payment_signature(headers: &HeaderMap) -> Result<PaymentSignatureEnvelope, X402Error> {
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

/// Adds the XLMP extension without conflating payment validity with proof
/// validity. The proof certificate is delivered as a separate resource or
/// extension only after independent verification.
pub fn with_xlmp_extension(
    mut required: PaymentRequired,
    extension: &XlmpPaymentExtension,
) -> Result<PaymentRequired, X402Error> {
    if extension.protocol != XLMP_VERSION {
        return Err(X402Error::InvalidExtension(format!(
            "expected protocol {XLMP_VERSION}"
        )));
    }
    extension
        .xlmp_message_id
        .validate()
        .map_err(|error| X402Error::InvalidExtension(error.to_string()))?;
    let _ = required
        .extensions
        .insert("xlmp".to_owned(), serde_json::to_value(extension)?);
    Ok(required)
}

#[deprecated(note = "use XlmpPaymentExtension")]
pub type XLemmaPaymentExtension = XlmpPaymentExtension;

#[deprecated(note = "use with_xlmp_extension")]
pub fn with_xlemma_extension(
    required: PaymentRequired,
    extension: &XlmpPaymentExtension,
) -> Result<PaymentRequired, X402Error> {
    with_xlmp_extension(required, extension)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use serde_json::json;
    use xlemma_core::TheoryId;

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

    #[test]
    fn payment_offer_binds_xlmp_without_becoming_research_consensus() {
        let extension = XlmpPaymentExtension {
            protocol: XLMP_VERSION.into(),
            xlmp_message_id: MessageId::derive(&"message").unwrap(),
            job_id: JobId::derive(&"job").unwrap(),
            researcher_id: ResearcherId::derive(&"researcher").unwrap(),
            claim_id: ClaimId::from_canonical_elaborated_type(
                &TheoryId::derive(&"theory").unwrap(),
                "claim",
            )
            .unwrap(),
            proof_id: None,
            artifact_commitment: "blake3:artifact".into(),
            compute_quote_id: ComputeQuoteId::derive(&"quote").unwrap(),
            required_verification_policy: PolicyId::derive(&"policy").unwrap(),
            model_policy: "provider-neutral".into(),
            rights_manifest_hash: "blake3:rights".into(),
            revenue_route_hash: "blake3:revenue".into(),
            delivery_mode: "public_bundle".into(),
            valid_until: Utc::now() + Duration::hours(1),
        };
        let required = PaymentRequired {
            x402_version: 2,
            error: "payment required".into(),
            resource: ResourceDescription {
                url: "/resource".into(),
                description: "test".into(),
                mime_type: "application/json".into(),
            },
            accepts: vec![],
            extensions: BTreeMap::new(),
        };
        let bound = with_xlmp_extension(required, &extension).unwrap();
        assert!(bound.extensions.contains_key("xlmp"));
        assert!(!bound.extensions.contains_key("xlemma"));
    }
}
