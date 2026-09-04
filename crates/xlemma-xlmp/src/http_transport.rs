//! Hardened HTTP transport adapter for canonical XLMP envelopes.

use crate::{AdapterError, TransportAdapter, XlmpEnvelope, XLMP_MEDIA_TYPE};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::{redirect::Policy as RedirectPolicy, StatusCode, Url};
use std::{collections::BTreeSet, time::Duration};
use thiserror::Error;
use xlemma_core::{canonical_json_bytes, ReceiptId, TransportReceipt};

const MAX_RESPONSE_BYTES: usize = 1_048_576;
const MAX_TRANSPORT_REFERENCE_BYTES: usize = 512;

pub struct HttpTransportConfig {
    pub endpoint: String,
    pub allowed_hosts: BTreeSet<String>,
    /// Kept out of receipts and error messages.
    pub bearer_token: Option<String>,
    pub timeout_seconds: u64,
}

pub trait TransportReceiptSigner: Send + Sync {
    fn sign(&self, signing_bytes: &[u8]) -> Result<String, HttpTransportError>;
}

pub struct HttpTransportAdapter<S> {
    endpoint: Url,
    bearer_token: Option<String>,
    client: reqwest::Client,
    signer: S,
}

impl<S: TransportReceiptSigner> HttpTransportAdapter<S> {
    pub fn new(config: HttpTransportConfig, signer: S) -> Result<Self, HttpTransportError> {
        let endpoint = Url::parse(&config.endpoint).map_err(|_| HttpTransportError::Endpoint)?;
        let host = endpoint.host_str().ok_or(HttpTransportError::Endpoint)?;
        if endpoint.scheme() != "https"
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.fragment().is_some()
            || config.allowed_hosts.is_empty()
            || !config.allowed_hosts.contains(host)
            || config.timeout_seconds == 0
            || config.bearer_token.as_ref().is_some_and(String::is_empty)
        {
            return Err(HttpTransportError::Endpoint);
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .redirect(RedirectPolicy::none())
            .build()
            .map_err(HttpTransportError::Http)?;
        Ok(Self {
            endpoint,
            bearer_token: config.bearer_token,
            client,
            signer,
        })
    }
}

#[async_trait]
impl<S: TransportReceiptSigner> TransportAdapter for HttpTransportAdapter<S> {
    async fn send(&self, envelope: &XlmpEnvelope) -> Result<TransportReceipt, AdapterError> {
        envelope
            .validate_integrity()
            .map_err(|_| adapter_error(HttpTransportError::Envelope))?;
        let body = canonical_json_bytes(envelope)
            .map_err(|_| adapter_error(HttpTransportError::Envelope))?;
        let mut request = self
            .client
            .post(self.endpoint.clone())
            .header(reqwest::header::CONTENT_TYPE, XLMP_MEDIA_TYPE)
            .body(body);
        if let Some(token) = &self.bearer_token {
            request = request.bearer_auth(token);
        }
        let mut response = request
            .send()
            .await
            .map_err(HttpTransportError::Http)
            .map_err(adapter_error)?;
        if response.status() != StatusCode::ACCEPTED {
            return Err(adapter_error(HttpTransportError::Status(response.status())));
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
        {
            return Err(adapter_error(HttpTransportError::ResponseTooLarge));
        }
        let request_reference = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .unwrap_or(envelope.message_id.as_str())
            .to_owned();
        if request_reference.is_empty() || request_reference.len() > MAX_TRANSPORT_REFERENCE_BYTES {
            return Err(adapter_error(HttpTransportError::Reference));
        }
        let mut response_bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(HttpTransportError::Http)
            .map_err(adapter_error)?
        {
            if response_bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
                return Err(adapter_error(HttpTransportError::ResponseTooLarge));
            }
            response_bytes.extend_from_slice(&chunk);
        }
        let accepted: XlmpEnvelope = serde_json::from_slice(&response_bytes)
            .map_err(HttpTransportError::Json)
            .map_err(adapter_error)?;
        if accepted != *envelope || accepted.validate_integrity().is_err() {
            return Err(adapter_error(HttpTransportError::Envelope));
        }
        let mut receipt = TransportReceipt {
            receipt_id: ReceiptId::derive(&"placeholder")
                .map_err(|_| adapter_error(HttpTransportError::Receipt))?,
            message_id: envelope.message_id.clone(),
            transport: "https".into(),
            destination: self.endpoint.origin().ascii_serialization(),
            delivered_at: Utc::now(),
            transport_reference: request_reference,
            signature: String::new(),
        };
        receipt.receipt_id = receipt
            .derive_receipt_id()
            .map_err(|_| adapter_error(HttpTransportError::Receipt))?;
        receipt.signature = self
            .signer
            .sign(
                &receipt
                    .signing_bytes()
                    .map_err(|_| adapter_error(HttpTransportError::Receipt))?,
            )
            .map_err(adapter_error)?;
        receipt
            .validate_integrity()
            .map_err(|_| adapter_error(HttpTransportError::Receipt))?;
        Ok(receipt)
    }
}

#[derive(Debug, Error)]
pub enum HttpTransportError {
    #[error("HTTP endpoint must be an allowlisted HTTPS origin without embedded credentials")]
    Endpoint,
    #[error("canonical XLMP envelope or returned MessageID is invalid")]
    Envelope,
    #[error("HTTP transport failed")]
    Http(#[source] reqwest::Error),
    #[error("XLMP HTTP endpoint returned {0}")]
    Status(StatusCode),
    #[error("XLMP HTTP response exceeded the byte limit")]
    ResponseTooLarge,
    #[error("XLMP HTTP response was not valid JSON")]
    Json(#[source] serde_json::Error),
    #[error("XLMP HTTP response carried an invalid transport reference")]
    Reference,
    #[error("transport receipt signing or integrity failed")]
    Receipt,
}

fn adapter_error(error: HttpTransportError) -> AdapterError {
    AdapterError {
        adapter: "xlmp-over-https".into(),
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSigner;

    impl TransportReceiptSigner for TestSigner {
        fn sign(&self, _signing_bytes: &[u8]) -> Result<String, HttpTransportError> {
            Ok("test-transport-signature".into())
        }
    }

    #[test]
    fn endpoint_policy_rejects_http_credentials_redirect_targets_and_unlisted_hosts() {
        let config = |endpoint: &str| HttpTransportConfig {
            endpoint: endpoint.into(),
            allowed_hosts: BTreeSet::from(["api.xlemma.example".into()]),
            bearer_token: Some("secret-not-logged".into()),
            timeout_seconds: 30,
        };
        assert!(HttpTransportAdapter::new(
            config("https://api.xlemma.example/xlmp/v1/messages"),
            TestSigner
        )
        .is_ok());
        assert!(HttpTransportAdapter::new(
            config("http://api.xlemma.example/xlmp/v1/messages"),
            TestSigner
        )
        .is_err());
        assert!(HttpTransportAdapter::new(
            config("https://attacker.example/xlmp/v1/messages"),
            TestSigner
        )
        .is_err());
        assert!(HttpTransportAdapter::new(
            config("https://user:secret@api.xlemma.example/xlmp/v1/messages"),
            TestSigner
        )
        .is_err());
    }
}
