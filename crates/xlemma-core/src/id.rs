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
    #[error(transparent)]
    Canonicalization(#[from] CanonicalizationError),
}

macro_rules! protocol_id {
    ($name:ident, $prefix:literal, $domain:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub const PREFIX: &'static str = $prefix;
            pub const DOMAIN: &'static str = $domain;

            pub fn derive<T: Serialize>(value: &T) -> Result<Self, IdError> {
                let digest = canonical_json_hash(Self::DOMAIN, value)?;
                Ok(Self(format!("{}{}", Self::PREFIX, hex::encode(digest))))
            }

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

protocol_id!(TheoryId, "xlt:blake3:", "theory-v1");
protocol_id!(ClaimId, "xlc:blake3:", "claim-v1");
protocol_id!(ProofId, "xlp:blake3:", "proof-v1");
protocol_id!(ArtifactId, "xla:blake3:", "artifact-v1");
protocol_id!(ReceiptId, "xlr:blake3:", "receipt-v1");
protocol_id!(LemmaId, "xll:blake3:", "lemma-v1");
protocol_id!(ResearcherId, "xlresearcher:blake3:", "researcher-v1");
protocol_id!(JobId, "xljob:blake3:", "job-v1");
protocol_id!(PolicyId, "xlpolicy:blake3:", "policy-v1");
protocol_id!(NodeId, "xlnode:blake3:", "node-v1");
protocol_id!(OperatorClusterId, "xloperator:blake3:", "operator-cluster-v1");
protocol_id!(RevenueEventId, "xlrevenue:blake3:", "revenue-event-v1");
protocol_id!(ComputeQuoteId, "xlquote:blake3:", "compute-quote-v1");

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ids_are_domain_separated() {
        let value = json!({"same": "content"});
        let claim = ClaimId::derive(&value).unwrap();
        let proof = ProofId::derive(&value).unwrap();
        assert_ne!(claim.as_str(), proof.as_str());
    }
}
