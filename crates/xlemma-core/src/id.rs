use crate::{canonical_json_hash, CanonicalizationError};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdError {
    #[error("identifier has an invalid prefix: expected {expected}, found {found}")]
    InvalidPrefix {
        expected: &'static str,
        found: String,
    },
    #[error("identifier digest is not 32-byte lowercase hex")]
    InvalidDigest,
    #[error("canonical identity component {0} must not be empty")]
    EmptyCanonicalComponent(&'static str),
    #[error(transparent)]
    Canonicalization(#[from] CanonicalizationError),
}

macro_rules! protocol_id_base {
    ($name:ident, $prefix:literal, $domain:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub const PREFIX: &'static str = $prefix;
            pub const DOMAIN: &'static str = $domain;

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn validate(&self) -> Result<(), IdError> {
                Self::from_str(&self.0).map(|_| ())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let Some(digest) = value.strip_prefix(Self::PREFIX) else {
                    let found = value.split(':').take(2).collect::<Vec<_>>().join(":");
                    return Err(IdError::InvalidPrefix {
                        expected: Self::PREFIX,
                        found,
                    });
                };
                if digest.len() != 64
                    || !digest.chars().all(|character| {
                        character.is_ascii_digit() || ('a'..='f').contains(&character)
                    })
                {
                    return Err(IdError::InvalidDigest);
                }
                Ok(Self(value.to_owned()))
            }
        }
    };
}

macro_rules! protocol_id {
    ($name:ident, $prefix:literal, $domain:literal) => {
        protocol_id_base!($name, $prefix, $domain);

        impl $name {
            pub fn derive<T: Serialize>(value: &T) -> Result<Self, IdError> {
                let digest = canonical_json_hash(Self::DOMAIN, value)?;
                Ok(Self(format!("{}{}", Self::PREFIX, hex::encode(digest))))
            }
        }
    };
}

protocol_id!(TheoryId, "xlt:blake3:", "theory-v1");
protocol_id_base!(ClaimId, "xlc:blake3:", "claim-v1");
protocol_id_base!(ProofId, "xlp:blake3:", "proof-v1");
protocol_id!(ArtifactId, "xla:blake3:", "artifact-v1");
protocol_id!(ReceiptId, "xlr:blake3:", "receipt-v1");
protocol_id!(LemmaId, "xll:blake3:", "lemma-v1");
protocol_id!(ResearcherId, "xlresearcher:blake3:", "researcher-v1");
protocol_id!(JobId, "xljob:blake3:", "job-v1");
protocol_id!(PolicyId, "xlpolicy:blake3:", "policy-v1");
protocol_id!(NodeId, "xlnode:blake3:", "node-v1");
protocol_id!(
    OperatorClusterId,
    "xloperator:blake3:",
    "operator-cluster-v1"
);
protocol_id!(RevenueEventId, "xlrevenue:blake3:", "revenue-event-v1");
protocol_id!(ComputeQuoteId, "xlquote:blake3:", "compute-quote-v1");
protocol_id!(MessageId, "xlmessage:blake3:", "xlmp-message-v1");
protocol_id!(CertificateId, "xlcert:blake3:", "poir-certificate-v1");
protocol_id!(ChallengeId, "xlchallenge:blake3:", "challenge-v1");
protocol_id!(QuarantineId, "xlquarantine:blake3:", "quarantine-v1");
protocol_id!(CreditId, "xlcredit:blake3:", "research-credit-v1");
protocol_id!(VaultId, "xlvault:blake3:", "research-vault-v1");
protocol_id!(DividendId, "xldividend:blake3:", "dependency-dividend-v1");
protocol_id!(LicenseId, "xllicense:blake3:", "license-v1");
protocol_id!(PublicationId, "xlpublication:blake3:", "publication-v1");

impl ClaimId {
    /// Derive a ClaimID from the canonical elaborated formal type under its
    /// TheoryID. This deliberately offers no source-text-only constructor.
    pub fn from_canonical_elaborated_type(
        theory_id: &TheoryId,
        canonical_elaborated_type: &str,
    ) -> Result<Self, IdError> {
        if canonical_elaborated_type.trim().is_empty() {
            return Err(IdError::EmptyCanonicalComponent(
                "canonical_elaborated_type",
            ));
        }
        #[derive(Serialize)]
        struct Identity<'a> {
            theory_id: &'a TheoryId,
            canonical_elaborated_type: &'a str,
        }

        let digest = canonical_json_hash(
            Self::DOMAIN,
            &Identity {
                theory_id,
                canonical_elaborated_type,
            },
        )?;
        Ok(Self(format!("{}{}", Self::PREFIX, hex::encode(digest))))
    }
}

impl ProofId {
    /// Derive a ProofID from the canonical checker-consumable proof object
    /// bound to its ClaimID.
    pub fn from_canonical_proof_object(
        claim_id: &ClaimId,
        canonical_proof_object: &str,
    ) -> Result<Self, IdError> {
        if canonical_proof_object.trim().is_empty() {
            return Err(IdError::EmptyCanonicalComponent("canonical_proof_object"));
        }
        #[derive(Serialize)]
        struct Identity<'a> {
            claim_id: &'a ClaimId,
            canonical_proof_object: &'a str,
        }

        let digest = canonical_json_hash(
            Self::DOMAIN,
            &Identity {
                claim_id,
                canonical_proof_object,
            },
        )?;
        Ok(Self(format!("{}{}", Self::PREFIX, hex::encode(digest))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ids_are_domain_separated() {
        let theory = TheoryId::derive(&json!({"same": "theory"})).unwrap();
        let claim = ClaimId::from_canonical_elaborated_type(&theory, "same content").unwrap();
        let proof = ProofId::from_canonical_proof_object(&claim, "same content").unwrap();
        assert_ne!(claim.as_str(), proof.as_str());
    }

    #[test]
    fn empty_formal_identity_components_are_rejected() {
        let theory = TheoryId::derive(&"theory").unwrap();
        assert!(matches!(
            ClaimId::from_canonical_elaborated_type(&theory, "  "),
            Err(IdError::EmptyCanonicalComponent(
                "canonical_elaborated_type"
            ))
        ));
        let claim = ClaimId::from_canonical_elaborated_type(&theory, "True").unwrap();
        assert!(matches!(
            ProofId::from_canonical_proof_object(&claim, ""),
            Err(IdError::EmptyCanonicalComponent("canonical_proof_object"))
        ));
    }
}
