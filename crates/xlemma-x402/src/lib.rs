//! x402 V2 payment-adapter objects for XLMP/1.
//!
//! x402 transports payment authorization; it does not define xLemma research
//! state, verification, rights, or consensus. This crate does not reimplement
//! chain-specific settlement. Use an audited SDK/facilitator behind
//! `PaymentFacilitator`.

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Duration, Utc};
use http::{HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Mutex,
};
use thiserror::Error;
use xlemma_core::{
    canonical_json_hash, Amount, ClaimId, ComputeQuoteId, JobId, MessageId, PaymentReceipt,
    PolicyId, ProofId, ResearcherId, XLMP_VERSION,
};
use xlemma_xlmp::{AdapterError, PaymentAdapter, PaymentAuthorization, PaymentInstruction};

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
    VariableResearchProverSearch,
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
        ResearchServiceKind::VariableResearchProverSearch | ResearchServiceKind::MeteredRepair => {
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
    #[error("payment instruction or adapter configuration is invalid")]
    InvalidInstruction,
    #[error("payment authorization has expired, was altered, or was already consumed")]
    InvalidAuthorization,
    #[error("payment facilitator returned a receipt that does not bind the authorization")]
    InvalidReceipt,
    #[error("payment adapter replay state is unavailable")]
    ReplayState,
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

#[derive(Clone, Debug)]
pub struct X402AdapterConfig {
    pub scheme: PaymentScheme,
    pub network: String,
    pub pay_to: String,
    pub authorization_timeout_seconds: u64,
}

#[derive(Clone, Debug)]
pub struct AuthorizedX402Payment {
    pub signature: PaymentSignatureEnvelope,
    /// Payer-controlled signature or custody attestation over the authorization.
    pub authorization_attestation: String,
}

#[async_trait]
pub trait X402Payer: Send + Sync {
    async fn authorize(
        &self,
        instruction: &PaymentInstruction,
        requirement: &PaymentRequirement,
    ) -> Result<AuthorizedX402Payment, X402Error>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredAuthorizationReference {
    signature: PaymentSignatureEnvelope,
    job_id: JobId,
    quote_id: ComputeQuoteId,
    payer: String,
    payee: String,
    scheme: PaymentScheme,
    network: String,
    authorized: Amount,
    payment_terms_root: String,
    expires_at: DateTime<Utc>,
    authorization_attestation: String,
}

/// Concrete XLMP `PaymentAdapter` over an x402 payer and facilitator. The
/// adapter consumes an authorization exactly once locally and rejects any
/// facilitator receipt that changes the job, parties, amounts, scheme, or
/// network. Durable deployments replace the local replay set with a
/// transactional authorization store.
pub struct X402PaymentAdapter<F, P> {
    config: X402AdapterConfig,
    facilitator: F,
    payer: P,
    authorizations: Mutex<AuthorizationState>,
}

#[derive(Default)]
struct AuthorizationState {
    issued: BTreeMap<String, String>,
    consumed_payments: BTreeSet<(String, String)>,
}

impl<F, P> X402PaymentAdapter<F, P> {
    pub fn new(config: X402AdapterConfig, facilitator: F, payer: P) -> Result<Self, X402Error> {
        if config.network.trim().is_empty()
            || config.pay_to.trim().is_empty()
            || config.authorization_timeout_seconds == 0
            || i64::try_from(config.authorization_timeout_seconds)
                .ok()
                .and_then(Duration::try_seconds)
                .and_then(|duration| Utc::now().checked_add_signed(duration))
                .is_none()
        {
            return Err(X402Error::InvalidInstruction);
        }
        Ok(Self {
            config,
            facilitator,
            payer,
            authorizations: Mutex::new(AuthorizationState::default()),
        })
    }
}

#[async_trait]
impl<F: PaymentFacilitator, P: X402Payer> PaymentAdapter for X402PaymentAdapter<F, P> {
    async fn authorize(
        &self,
        instruction: PaymentInstruction,
    ) -> Result<PaymentAuthorization, AdapterError> {
        if instruction.job_id.validate().is_err()
            || instruction.quote_id.validate().is_err()
            || instruction.payer.trim().is_empty()
            || instruction.payee != self.config.pay_to
            || instruction.maximum_authorization.units == 0
            || instruction.maximum_authorization.asset.trim().is_empty()
            || instruction.payment_terms_root.trim().is_empty()
            || instruction.valid_until <= Utc::now()
        {
            return Err(adapter_error(X402Error::InvalidInstruction));
        }
        let requirement = PaymentRequirement {
            scheme: self.config.scheme,
            network: self.config.network.clone(),
            amount: instruction.maximum_authorization.units.to_string(),
            asset: instruction.maximum_authorization.asset.clone(),
            pay_to: self.config.pay_to.clone(),
            max_timeout_seconds: self.config.authorization_timeout_seconds,
            extra: BTreeMap::from([
                (
                    "jobId".into(),
                    Value::String(instruction.job_id.to_string()),
                ),
                (
                    "quoteId".into(),
                    Value::String(instruction.quote_id.to_string()),
                ),
                (
                    "paymentTermsRoot".into(),
                    Value::String(instruction.payment_terms_root.clone()),
                ),
            ]),
        };
        let authorized = self
            .payer
            .authorize(&instruction, &requirement)
            .await
            .map_err(adapter_error)?;
        if authorized.authorization_attestation.trim().is_empty()
            || authorized.signature.x402_version != 2
            || authorized.signature.scheme != self.config.scheme
            || authorized.signature.network != self.config.network
            || authorized.signature.payment_identifier.trim().is_empty()
        {
            return Err(adapter_error(X402Error::InvalidAuthorization));
        }
        self.facilitator
            .verify_authorization(&requirement, &authorized.signature)
            .await
            .map_err(adapter_error)?;
        let timeout = i64::try_from(self.config.authorization_timeout_seconds)
            .ok()
            .and_then(Duration::try_seconds)
            .and_then(|duration| Utc::now().checked_add_signed(duration))
            .ok_or_else(|| adapter_error(X402Error::InvalidInstruction))?;
        let expires_at = instruction.valid_until.min(timeout);
        if expires_at <= Utc::now() {
            return Err(adapter_error(X402Error::InvalidAuthorization));
        }
        let reference = StoredAuthorizationReference {
            signature: authorized.signature,
            job_id: instruction.job_id.clone(),
            quote_id: instruction.quote_id,
            payer: instruction.payer.clone(),
            payee: instruction.payee.clone(),
            scheme: self.config.scheme,
            network: self.config.network.clone(),
            authorized: instruction.maximum_authorization.clone(),
            payment_terms_root: instruction.payment_terms_root,
            expires_at,
            authorization_attestation: authorized.authorization_attestation,
        };
        let reference_bytes = serde_json::to_vec(&reference).map_err(X402Error::Json);
        let reference_bytes = reference_bytes.map_err(adapter_error)?;
        let authorization_id = format!(
            "x402auth:{}",
            hex_digest(
                canonical_json_hash("x402-payment-authorization-v1", &reference)
                    .map_err(|_| adapter_error(X402Error::InvalidAuthorization))?
            )
        );
        let authorization_reference = STANDARD.encode(reference_bytes);
        self.authorizations
            .lock()
            .map_err(|_| adapter_error(X402Error::ReplayState))?
            .issued
            .insert(authorization_id.clone(), authorization_reference.clone());
        Ok(PaymentAuthorization {
            authorization_id,
            job_id: instruction.job_id,
            adapter: "x402".into(),
            authorized: instruction.maximum_authorization,
            authorization_reference,
            expires_at,
            signature: reference.authorization_attestation,
        })
    }

    async fn settle(
        &self,
        authorization: PaymentAuthorization,
        actual_charge: Amount,
    ) -> Result<PaymentReceipt, AdapterError> {
        authorization
            .authorized
            .ensure_compatible(&actual_charge)
            .map_err(|_| adapter_error(X402Error::InvalidAuthorization))?;
        if authorization.adapter != "x402"
            || authorization.authorization_id.trim().is_empty()
            || authorization.authorization_reference.trim().is_empty()
            || authorization.signature.trim().is_empty()
            || authorization.expires_at <= Utc::now()
            || actual_charge.units > authorization.authorized.units
            || (self.config.scheme == PaymentScheme::Exact
                && actual_charge.units != authorization.authorized.units)
        {
            return Err(adapter_error(X402Error::InvalidAuthorization));
        }
        let reference_bytes = STANDARD
            .decode(&authorization.authorization_reference)
            .map_err(X402Error::Base64)
            .map_err(adapter_error)?;
        let reference: StoredAuthorizationReference = serde_json::from_slice(&reference_bytes)
            .map_err(X402Error::Json)
            .map_err(adapter_error)?;
        let expected_id = format!(
            "x402auth:{}",
            hex_digest(
                canonical_json_hash("x402-payment-authorization-v1", &reference)
                    .map_err(|_| adapter_error(X402Error::InvalidAuthorization))?
            )
        );
        if authorization.authorization_id != expected_id
            || authorization.job_id != reference.job_id
            || authorization.authorized != reference.authorized
            || authorization.expires_at != reference.expires_at
            || authorization.signature != reference.authorization_attestation
            || reference.scheme != self.config.scheme
            || reference.network != self.config.network
            || reference.payee != self.config.pay_to
            || reference.payment_terms_root.trim().is_empty()
            || reference.signature.x402_version != 2
            || reference.signature.scheme != reference.scheme
            || reference.signature.network != reference.network
            || reference.signature.payment_identifier.trim().is_empty()
        {
            return Err(adapter_error(X402Error::InvalidAuthorization));
        }
        // A content hash is not authorization: callers can recompute it after
        // changing an expiry or payment binding. Require an exact issued record.
        if self
            .authorizations
            .lock()
            .map_err(|_| adapter_error(X402Error::ReplayState))?
            .issued
            .get(&authorization.authorization_id)
            != Some(&authorization.authorization_reference)
        {
            return Err(adapter_error(X402Error::InvalidAuthorization));
        }
        let requirement = PaymentRequirement {
            scheme: reference.scheme,
            network: reference.network.clone(),
            amount: reference.authorized.units.to_string(),
            asset: reference.authorized.asset.clone(),
            pay_to: reference.payee.clone(),
            max_timeout_seconds: self.config.authorization_timeout_seconds,
            extra: BTreeMap::from([
                ("jobId".into(), Value::String(reference.job_id.to_string())),
                (
                    "quoteId".into(),
                    Value::String(reference.quote_id.to_string()),
                ),
                (
                    "paymentTermsRoot".into(),
                    Value::String(reference.payment_terms_root.clone()),
                ),
            ]),
        };
        self.facilitator
            .verify_authorization(&requirement, &reference.signature)
            .await
            .map_err(adapter_error)?;
        {
            let mut state = self
                .authorizations
                .lock()
                .map_err(|_| adapter_error(X402Error::ReplayState))?;
            // Rewrapping the same facilitator payment must never create a
            // second settlement attempt, including concurrent requests.
            if !state.consumed_payments.insert((
                reference.network.clone(),
                reference.signature.payment_identifier.clone(),
            )) {
                return Err(adapter_error(X402Error::InvalidAuthorization));
            }
        }

        let result = self
            .facilitator
            .settle(SettlementRequest {
                job_id: authorization.job_id.clone(),
                maximum_authorization: authorization.authorized.clone(),
                actual_charge: actual_charge.clone(),
                signature: reference.signature.clone(),
                pay_to: reference.payee.clone(),
            })
            .await;
        let receipt = match result {
            Ok(receipt) => receipt,
            Err(error) => {
                // A transport error may follow a successful external charge.
                // Keep the attempt consumed until external reconciliation.
                return Err(adapter_error(error));
            }
        };
        let receipt_valid = receipt.validate_integrity().is_ok()
            && receipt.job_id == authorization.job_id
            && receipt.payment_identifier == reference.signature.payment_identifier
            && receipt.scheme == scheme_name(self.config.scheme)
            && receipt.network == reference.network
            && receipt.payer == reference.payer
            && receipt.payee == reference.payee
            && receipt.authorized == authorization.authorized
            && receipt.settled == actual_charge;
        if !receipt_valid {
            return Err(adapter_error(X402Error::InvalidReceipt));
        }
        Ok(receipt)
    }
}

fn scheme_name(scheme: PaymentScheme) -> &'static str {
    match scheme {
        PaymentScheme::Exact => "exact",
        PaymentScheme::Upto => "upto",
        PaymentScheme::BatchSettlement => "batch-settlement",
    }
}

fn hex_digest(digest: [u8; 32]) -> String {
    use std::fmt::Write as _;

    digest.iter().fold(
        String::with_capacity(digest.len() * 2),
        |mut encoded, byte| {
            write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
            encoded
        },
    )
}

fn adapter_error(error: X402Error) -> AdapterError {
    AdapterError {
        adapter: "x402".into(),
        reason: error.to_string(),
    }
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

    struct TestPayer;

    #[async_trait]
    impl X402Payer for TestPayer {
        async fn authorize(
            &self,
            _instruction: &PaymentInstruction,
            requirement: &PaymentRequirement,
        ) -> Result<AuthorizedX402Payment, X402Error> {
            Ok(AuthorizedX402Payment {
                signature: PaymentSignatureEnvelope {
                    x402_version: 2,
                    scheme: requirement.scheme,
                    network: requirement.network.clone(),
                    payload: json!({"authorization": "test-only"}),
                    payment_identifier: "payment-attempt-1".into(),
                },
                authorization_attestation: "payer-signature".into(),
            })
        }
    }

    struct TestFacilitator;

    #[async_trait]
    impl PaymentFacilitator for TestFacilitator {
        async fn verify_authorization(
            &self,
            requirement: &PaymentRequirement,
            signature: &PaymentSignatureEnvelope,
        ) -> Result<(), X402Error> {
            if signature.network != requirement.network
                || signature.scheme != requirement.scheme
                || signature.payment_identifier.is_empty()
            {
                return Err(X402Error::Verification("binding mismatch".into()));
            }
            Ok(())
        }

        async fn settle(&self, request: SettlementRequest) -> Result<PaymentReceipt, X402Error> {
            let mut receipt = PaymentReceipt {
                receipt_id: xlemma_core::ReceiptId::derive(&"placeholder").unwrap(),
                job_id: request.job_id,
                payment_identifier: request.signature.payment_identifier,
                scheme: scheme_name(request.signature.scheme).into(),
                network: request.signature.network,
                payer: "did:key:payer".into(),
                payee: request.pay_to,
                authorized: request.maximum_authorization,
                settled: request.actual_charge,
                settlement_reference: "eip155:8453:0xsettlement".into(),
                settled_at: Utc::now(),
                facilitator_signature: "facilitator-signature".into(),
            };
            receipt.receipt_id = receipt.derive_receipt_id().unwrap();
            Ok(receipt)
        }
    }

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

    #[tokio::test]
    async fn concrete_adapter_settles_actual_usage_once_and_preserves_payment_separation() {
        let adapter = X402PaymentAdapter::new(
            X402AdapterConfig {
                scheme: PaymentScheme::Upto,
                network: "eip155:8453".into(),
                pay_to: "did:key:node".into(),
                authorization_timeout_seconds: 300,
            },
            TestFacilitator,
            TestPayer,
        )
        .unwrap();
        let instruction = PaymentInstruction {
            job_id: JobId::derive(&"job").unwrap(),
            quote_id: ComputeQuoteId::derive(&"quote").unwrap(),
            payer: "did:key:payer".into(),
            payee: "did:key:node".into(),
            maximum_authorization: Amount::new(100_000, "USDC", 6),
            payment_terms_root: "blake3:payment-terms".into(),
            valid_until: Utc::now() + Duration::minutes(10),
        };
        let authorization = adapter.authorize(instruction.clone()).await.unwrap();
        let mut forged = authorization.clone();
        let mut reference: StoredAuthorizationReference =
            serde_json::from_slice(&STANDARD.decode(&forged.authorization_reference).unwrap())
                .unwrap();
        reference.expires_at += Duration::hours(1);
        forged.expires_at = reference.expires_at;
        forged.authorization_reference = STANDARD.encode(serde_json::to_vec(&reference).unwrap());
        forged.authorization_id = format!(
            "x402auth:{}",
            hex_digest(canonical_json_hash("x402-payment-authorization-v1", &reference).unwrap())
        );
        assert!(adapter
            .settle(forged, Amount::new(72_000, "USDC", 6))
            .await
            .is_err());
        let mut altered = authorization.clone();
        altered.job_id = JobId::derive(&"different-job").unwrap();
        assert!(adapter
            .settle(altered, Amount::new(72_000, "USDC", 6))
            .await
            .is_err());
        let receipt = adapter
            .settle(authorization.clone(), Amount::new(72_000, "USDC", 6))
            .await
            .unwrap();
        assert_eq!(receipt.settled.units, 72_000);
        assert_eq!(receipt.authorized.units, 100_000);
        assert!(receipt.validate_integrity().is_ok());
        assert!(adapter
            .settle(authorization, Amount::new(72_000, "USDC", 6))
            .await
            .is_err());
        // The payer returned the same payment identifier in a new wrapper.
        let reissued = adapter.authorize(instruction).await.unwrap();
        assert!(adapter
            .settle(reissued, Amount::new(72_000, "USDC", 6))
            .await
            .is_err());
    }

    struct UncertainFacilitator(std::sync::atomic::AtomicUsize);

    #[async_trait]
    impl PaymentFacilitator for UncertainFacilitator {
        async fn verify_authorization(
            &self,
            requirement: &PaymentRequirement,
            signature: &PaymentSignatureEnvelope,
        ) -> Result<(), X402Error> {
            TestFacilitator
                .verify_authorization(requirement, signature)
                .await
        }
        async fn settle(&self, _request: SettlementRequest) -> Result<PaymentReceipt, X402Error> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(X402Error::Settlement(
                "response lost after external charge".into(),
            ))
        }
    }

    #[tokio::test]
    async fn uncertain_external_settlement_cannot_be_retried() {
        let adapter = X402PaymentAdapter::new(
            X402AdapterConfig {
                scheme: PaymentScheme::Upto,
                network: "eip155:8453".into(),
                pay_to: "did:key:node".into(),
                authorization_timeout_seconds: 300,
            },
            UncertainFacilitator(std::sync::atomic::AtomicUsize::new(0)),
            TestPayer,
        )
        .unwrap();
        let authorization = adapter
            .authorize(PaymentInstruction {
                job_id: JobId::derive(&"job").unwrap(),
                quote_id: ComputeQuoteId::derive(&"quote").unwrap(),
                payer: "did:key:payer".into(),
                payee: "did:key:node".into(),
                maximum_authorization: Amount::new(100, "USDC", 6),
                payment_terms_root: "blake3:terms".into(),
                valid_until: Utc::now() + Duration::minutes(10),
            })
            .await
            .unwrap();
        for _ in 0..2 {
            assert!(adapter
                .settle(authorization.clone(), Amount::new(50, "USDC", 6))
                .await
                .is_err());
        }
        assert_eq!(
            adapter
                .facilitator
                .0
                .load(std::sync::atomic::Ordering::SeqCst),
            1
        );
    }

    #[test]
    fn unrepresentable_authorization_timeout_is_rejected_without_panicking() {
        for timeout in [u64::MAX, i64::MAX as u64] {
            assert!(X402PaymentAdapter::new(
                X402AdapterConfig {
                    scheme: PaymentScheme::Upto,
                    network: "eip155:8453".into(),
                    pay_to: "did:key:node".into(),
                    authorization_timeout_seconds: timeout,
                },
                TestFacilitator,
                TestPayer
            )
            .is_err());
        }
    }
}
