//! Domain-separated signing envelopes and replay protection.
//!
//! Concrete key custody and signature algorithms are deployment adapters. The
//! protocol digest and replay semantics remain stable across those adapters.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;
use xlemma_core::{canonical_json_hash, CanonicalizationError};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureDomain {
    pub protocol: String,
    pub protocol_version: String,
    pub network: String,
    pub verifying_contract_or_service: String,
    pub purpose: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningMetadata {
    pub signer: String,
    pub key_id: String,
    pub nonce: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedEnvelope<T> {
    pub domain: SignatureDomain,
    pub metadata: SigningMetadata,
    pub payload: T,
    pub digest: String,
    pub signature: String,
}

#[derive(Serialize)]
struct SigningMaterial<'a, T> {
    domain: &'a SignatureDomain,
    metadata: &'a SigningMetadata,
    payload: &'a T,
}

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("signing envelope is expired or not yet valid")]
    InvalidTime,
    #[error("signing nonce has already been consumed")]
    Replay,
    #[error("signing digest does not match the envelope")]
    DigestMismatch,
    #[error("signature verification failed")]
    InvalidSignature,
    #[error("signature domain contains an empty required field")]
    InvalidDomain,
    #[error("signing metadata contains an empty signer, key, or nonce")]
    InvalidMetadata,
    #[error(transparent)]
    Canonicalization(#[from] CanonicalizationError),
}

pub trait ProtocolSigner {
    fn signer_id(&self) -> &str;
    fn key_id(&self) -> &str;
    fn sign_digest(&self, digest: &[u8; 32]) -> Result<String, CryptoError>;
}

pub trait ProtocolVerifier {
    fn verify_digest(
        &self,
        signer: &str,
        key_id: &str,
        digest: &[u8; 32],
        signature: &str,
    ) -> Result<(), CryptoError>;
}

fn validate_domain_and_metadata(
    domain: &SignatureDomain,
    metadata: &SigningMetadata,
) -> Result<(), CryptoError> {
    if domain.protocol.is_empty()
        || domain.protocol_version.is_empty()
        || domain.network.is_empty()
        || domain.verifying_contract_or_service.is_empty()
        || domain.purpose.is_empty()
    {
        return Err(CryptoError::InvalidDomain);
    }
    if metadata.signer.is_empty() || metadata.key_id.is_empty() || metadata.nonce.is_empty() {
        return Err(CryptoError::InvalidMetadata);
    }
    if metadata.expires_at <= metadata.issued_at {
        return Err(CryptoError::InvalidTime);
    }
    Ok(())
}

pub fn signing_digest<T: Serialize>(
    domain: &SignatureDomain,
    metadata: &SigningMetadata,
    payload: &T,
) -> Result<[u8; 32], CryptoError> {
    Ok(canonical_json_hash(
        "signed-envelope-v1",
        &SigningMaterial {
            domain,
            metadata,
            payload,
        },
    )?)
}

pub fn create_envelope<T: Serialize>(
    signer: &impl ProtocolSigner,
    domain: SignatureDomain,
    mut metadata: SigningMetadata,
    payload: T,
) -> Result<SignedEnvelope<T>, CryptoError> {
    metadata.signer = signer.signer_id().to_owned();
    metadata.key_id = signer.key_id().to_owned();
    validate_domain_and_metadata(&domain, &metadata)?;
    let digest = signing_digest(&domain, &metadata, &payload)?;
    let signature = signer.sign_digest(&digest)?;
    Ok(SignedEnvelope {
        domain,
        metadata,
        payload,
        digest: format!("blake3:{}", hex_digest(&digest)),
        signature,
    })
}

pub fn verify_envelope<T: Serialize>(
    verifier: &impl ProtocolVerifier,
    envelope: &SignedEnvelope<T>,
    now: DateTime<Utc>,
) -> Result<(), CryptoError> {
    validate_domain_and_metadata(&envelope.domain, &envelope.metadata)?;
    if now < envelope.metadata.issued_at || now >= envelope.metadata.expires_at {
        return Err(CryptoError::InvalidTime);
    }
    let digest = signing_digest(&envelope.domain, &envelope.metadata, &envelope.payload)?;
    let expected = format!("blake3:{}", hex_digest(&digest));
    if envelope.digest != expected {
        return Err(CryptoError::DigestMismatch);
    }
    verifier.verify_digest(
        &envelope.metadata.signer,
        &envelope.metadata.key_id,
        &digest,
        &envelope.signature,
    )
}

/// In-memory reference replay guard. Production services require a durable,
/// transactionally consumed nonce store shared across replicas.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ReplayKey {
    protocol: String,
    protocol_version: String,
    network: String,
    verifying_contract_or_service: String,
    purpose: String,
    signer: String,
    key_id: String,
    nonce: String,
}

#[derive(Default)]
pub struct ReplayGuard {
    consumed: BTreeSet<ReplayKey>,
}

impl ReplayGuard {
    pub fn consume<T: Serialize>(
        &mut self,
        verifier: &impl ProtocolVerifier,
        envelope: &SignedEnvelope<T>,
        now: DateTime<Utc>,
    ) -> Result<(), CryptoError> {
        verify_envelope(verifier, envelope, now)?;
        let key = ReplayKey {
            protocol: envelope.domain.protocol.clone(),
            protocol_version: envelope.domain.protocol_version.clone(),
            network: envelope.domain.network.clone(),
            verifying_contract_or_service: envelope
                .domain
                .verifying_contract_or_service
                .clone(),
            purpose: envelope.domain.purpose.clone(),
            signer: envelope.metadata.signer.clone(),
            key_id: envelope.metadata.key_id.clone(),
            nonce: envelope.metadata.nonce.clone(),
        };
        if !self.consumed.insert(key) {
            return Err(CryptoError::Replay);
        }
        Ok(())
    }
}

fn hex_digest(digest: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    struct TestKey;

    impl ProtocolSigner for TestKey {
        fn signer_id(&self) -> &str {
            "did:test:signer"
        }

        fn key_id(&self) -> &str {
            "key-1"
        }

        fn sign_digest(&self, digest: &[u8; 32]) -> Result<String, CryptoError> {
            Ok(format!("test:{}", hex_digest(digest)))
        }
    }

    impl ProtocolVerifier for TestKey {
        fn verify_digest(
            &self,
            signer: &str,
            key_id: &str,
            digest: &[u8; 32],
            signature: &str,
        ) -> Result<(), CryptoError> {
            let expected = format!("test:{}", hex_digest(digest));
            if signer == "did:test:signer" && key_id == "key-1" && signature == expected {
                Ok(())
            } else {
                Err(CryptoError::InvalidSignature)
            }
        }
    }

    fn domain(purpose: &str) -> SignatureDomain {
        SignatureDomain {
            protocol: "xlemma".into(),
            protocol_version: "0.2".into(),
            network: "test".into(),
            verifying_contract_or_service: "service-a".into(),
            purpose: purpose.into(),
        }
    }

    #[test]
    fn digest_is_domain_separated() {
        let now = Utc::now();
        let metadata = SigningMetadata {
            signer: "did:test:signer".into(),
            key_id: "key-1".into(),
            nonce: "nonce-1".into(),
            issued_at: now,
            expires_at: now + Duration::minutes(5),
        };
        let payload = "same payload";
        assert_ne!(
            signing_digest(&domain("observation"), &metadata, &payload).unwrap(),
            signing_digest(&domain("payment"), &metadata, &payload).unwrap()
        );
    }

    #[test]
    fn replay_guard_consumes_nonce_once() {
        let now = Utc::now();
        let envelope = create_envelope(
            &TestKey,
            domain("observation"),
            SigningMetadata {
                signer: String::new(),
                key_id: String::new(),
                nonce: "nonce-1".into(),
                issued_at: now - Duration::seconds(1),
                expires_at: now + Duration::minutes(5),
            },
            "payload",
        )
        .unwrap();
        let mut guard = ReplayGuard::default();
        guard.consume(&TestKey, &envelope, now).unwrap();
        assert!(matches!(
            guard.consume(&TestKey, &envelope, now),
            Err(CryptoError::Replay)
        ));
    }

    #[test]
    fn nonce_scope_includes_the_full_signature_domain() {
        let now = Utc::now();
        let metadata = SigningMetadata {
            signer: String::new(),
            key_id: String::new(),
            nonce: "shared-nonce".into(),
            issued_at: now - Duration::seconds(1),
            expires_at: now + Duration::minutes(5),
        };
        let first = create_envelope(&TestKey, domain("observation"), metadata.clone(), "payload")
            .unwrap();
        let mut second_domain = domain("observation");
        second_domain.verifying_contract_or_service = "service-b".into();
        let second = create_envelope(&TestKey, second_domain, metadata, "payload").unwrap();

        let mut guard = ReplayGuard::default();
        guard.consume(&TestKey, &first, now).unwrap();
        guard.consume(&TestKey, &second, now).unwrap();
    }

    #[test]
    fn empty_nonce_is_rejected() {
        let now = Utc::now();
        let result = create_envelope(
            &TestKey,
            domain("observation"),
            SigningMetadata {
                signer: String::new(),
                key_id: String::new(),
                nonce: String::new(),
                issued_at: now,
                expires_at: now + Duration::minutes(5),
            },
            "payload",
        );
        assert!(matches!(result, Err(CryptoError::InvalidMetadata)));
    }

}
