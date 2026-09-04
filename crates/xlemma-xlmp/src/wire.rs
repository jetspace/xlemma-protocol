use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use xlemma_core::{
    canonical_json_bytes, ArtifactId, CanonicalizationError, CertificateId, Challenge, ClaimId,
    ClaimManifest, ComputeQuoteReceipt, IdError, JobId, MessageId, NodeId, ObservationReceipt,
    OperatorClusterId, PoIRCertificate, PolicyId, ProofId, ProofManifest, ReceiptId, ResearcherId,
    RevenueEvent, VerificationJob, XLMP_MAJOR_VERSION, XLMP_PROTOCOL,
};

use crate::XLMP_SIGNATURE_DOMAIN;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageKind {
    #[serde(rename = "XLMP_CLAIM")]
    Claim,
    #[serde(rename = "XLMP_COMMIT")]
    Commit,
    #[serde(rename = "XLMP_COMPUTE_QUOTE")]
    ComputeQuote,
    #[serde(rename = "XLMP_PROOF_CANDIDATE")]
    ProofCandidate,
    #[serde(rename = "XLMP_VERIFY_REQUEST")]
    VerifyRequest,
    #[serde(rename = "XLMP_OBSERVATION_COMMIT")]
    ObservationCommit,
    #[serde(rename = "XLMP_OBSERVATION_REVEAL")]
    ObservationReveal,
    #[serde(rename = "XLMP_CERTIFICATE")]
    Certificate,
    #[serde(rename = "XLMP_CHALLENGE")]
    Challenge,
    #[serde(rename = "XLMP_FINALIZE")]
    Finalize,
    #[serde(rename = "XLMP_REVENUE")]
    Revenue,
    #[serde(rename = "XLMP_REVALIDATE")]
    Revalidate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimMessage {
    pub claim_id: ClaimId,
    pub claim: ClaimManifest,
    pub contribution_manifest_hash: String,
    pub rights_manifest_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMessage {
    pub job_id: JobId,
    pub researcher_id: ResearcherId,
    pub claim_id: ClaimId,
    pub commitment_root: String,
    pub verification_policy_id: PolicyId,
    pub reveal_deadline: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComputeQuoteMessage {
    pub quote: ComputeQuoteReceipt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofCandidateMessage {
    pub job_id: JobId,
    pub proof_id: ProofId,
    pub proof: ProofManifest,
    pub artifact_id: ArtifactId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyRequestMessage {
    pub job: VerificationJob,
    pub exact_challenge_hash: String,
    pub dependency_root: String,
    pub axiom_policy_id: PolicyId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationCommitMessage {
    pub job_id: JobId,
    pub receipt_id: ReceiptId,
    pub node_id: NodeId,
    pub operator_cluster_id: OperatorClusterId,
    pub commitment: String,
    pub committed_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationRevealMessage {
    pub observation: ObservationReceipt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateMessage {
    pub certificate: PoIRCertificate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChallengeMessage {
    pub challenge: Challenge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalizeMessage {
    pub certificate_id: CertificateId,
    pub claim_id: ClaimId,
    pub finalization_root: String,
    pub finalized_at: DateTime<Utc>,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueMessage {
    pub event: RevenueEvent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevalidateMessage {
    pub certificate_id: CertificateId,
    pub claim_id: ClaimId,
    pub prior_observation_receipt_ids: Vec<ReceiptId>,
    pub verification_policy_id: PolicyId,
    pub reason: String,
    pub requested_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum XlmpMessage {
    #[serde(rename = "XLMP_CLAIM")]
    Claim(ClaimMessage),
    #[serde(rename = "XLMP_COMMIT")]
    Commit(CommitMessage),
    #[serde(rename = "XLMP_COMPUTE_QUOTE")]
    ComputeQuote(ComputeQuoteMessage),
    #[serde(rename = "XLMP_PROOF_CANDIDATE")]
    ProofCandidate(ProofCandidateMessage),
    #[serde(rename = "XLMP_VERIFY_REQUEST")]
    VerifyRequest(VerifyRequestMessage),
    #[serde(rename = "XLMP_OBSERVATION_COMMIT")]
    ObservationCommit(ObservationCommitMessage),
    #[serde(rename = "XLMP_OBSERVATION_REVEAL")]
    ObservationReveal(ObservationRevealMessage),
    #[serde(rename = "XLMP_CERTIFICATE")]
    Certificate(CertificateMessage),
    #[serde(rename = "XLMP_CHALLENGE")]
    Challenge(ChallengeMessage),
    #[serde(rename = "XLMP_FINALIZE")]
    Finalize(FinalizeMessage),
    #[serde(rename = "XLMP_REVENUE")]
    Revenue(RevenueMessage),
    #[serde(rename = "XLMP_REVALIDATE")]
    Revalidate(RevalidateMessage),
}

impl XlmpMessage {
    pub fn kind(&self) -> MessageKind {
        match self {
            Self::Claim(_) => MessageKind::Claim,
            Self::Commit(_) => MessageKind::Commit,
            Self::ComputeQuote(_) => MessageKind::ComputeQuote,
            Self::ProofCandidate(_) => MessageKind::ProofCandidate,
            Self::VerifyRequest(_) => MessageKind::VerifyRequest,
            Self::ObservationCommit(_) => MessageKind::ObservationCommit,
            Self::ObservationReveal(_) => MessageKind::ObservationReveal,
            Self::Certificate(_) => MessageKind::Certificate,
            Self::Challenge(_) => MessageKind::Challenge,
            Self::Finalize(_) => MessageKind::Finalize,
            Self::Revenue(_) => MessageKind::Revenue,
            Self::Revalidate(_) => MessageKind::Revalidate,
        }
    }

    fn validate_ids(&self) -> Result<(), IdError> {
        match self {
            Self::Claim(message) => {
                message.claim_id.validate()?;
                message.claim.theory_id.validate()
            }
            Self::Commit(message) => {
                message.job_id.validate()?;
                message.researcher_id.validate()?;
                message.claim_id.validate()?;
                message.verification_policy_id.validate()
            }
            Self::ComputeQuote(message) => {
                message.quote.quote_id.validate()?;
                message.quote.job_id.validate()?;
                message.quote.policy_id.validate()
            }
            Self::ProofCandidate(message) => {
                message.job_id.validate()?;
                message.proof_id.validate()?;
                message.proof.claim_id.validate()?;
                message.artifact_id.validate()
            }
            Self::VerifyRequest(message) => {
                message.job.job_id.validate()?;
                message.job.claim_id.validate()?;
                message.job.theory_id.validate()?;
                message.job.artifact_id.validate()?;
                message.axiom_policy_id.validate()
            }
            Self::ObservationCommit(message) => {
                message.job_id.validate()?;
                message.receipt_id.validate()?;
                message.node_id.validate()?;
                message.operator_cluster_id.validate()
            }
            Self::ObservationReveal(message) => {
                message.observation.receipt_id.validate()?;
                message.observation.job_id.validate()?;
                message.observation.node_id.validate()?;
                message.observation.operator_cluster_id.validate()
            }
            Self::Certificate(message) => {
                message.certificate.certificate_id.validate()?;
                message.certificate.job_id.validate()?;
                message.certificate.claim_id.validate()?;
                message.certificate.proof_id.validate()
            }
            Self::Challenge(message) => {
                message.challenge.challenge_id.validate()?;
                message.challenge.certificate_id.validate()
            }
            Self::Finalize(message) => {
                message.certificate_id.validate()?;
                message.claim_id.validate()
            }
            Self::Revenue(message) => {
                message.event.revenue_event_id.validate()?;
                message.event.claim_id.validate()
            }
            Self::Revalidate(message) => {
                message.certificate_id.validate()?;
                message.claim_id.validate()?;
                message.verification_policy_id.validate()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct XlmpEnvelope {
    pub protocol: String,
    pub version: u16,
    pub message_id: MessageId,
    pub correlation_id: Option<MessageId>,
    pub sender: String,
    pub sent_at: DateTime<Utc>,
    pub message: XlmpMessage,
    pub signature: String,
}

#[derive(Serialize)]
struct MessageIdentity<'a> {
    protocol: &'a str,
    version: u16,
    correlation_id: &'a Option<MessageId>,
    sender: &'a str,
    sent_at: &'a DateTime<Utc>,
    message: &'a XlmpMessage,
}

#[derive(Serialize)]
struct SigningMaterial<'a> {
    domain: &'static str,
    message_id: &'a MessageId,
    identity: MessageIdentity<'a>,
}

#[derive(Debug, Error)]
pub enum XlmpError {
    #[error("unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    #[error("unsupported XLMP major version: {0}")]
    UnsupportedVersion(u16),
    #[error("XLMP sender must not be empty")]
    EmptySender,
    #[error("XLMP envelope signature must not be empty")]
    EmptySignature,
    #[error("XLMP message identifier does not match its canonical content")]
    MessageIdMismatch,
    #[error("XLMP claim identifier does not match its canonical formal identity")]
    ClaimIdMismatch,
    #[error("XLMP proof identifier does not match its canonical proof identity")]
    ProofIdMismatch,
    #[error("XLMP observation reveal does not match its prior commitment")]
    ObservationCommitMismatch,
    #[error(transparent)]
    InvalidId(#[from] IdError),
}

impl XlmpEnvelope {
    pub fn new(
        correlation_id: Option<MessageId>,
        sender: impl Into<String>,
        sent_at: DateTime<Utc>,
        message: XlmpMessage,
        signature: impl Into<String>,
    ) -> Result<Self, XlmpError> {
        let sender = sender.into();
        let signature = signature.into();
        let message_id = derive_message_id(&correlation_id, &sender, &sent_at, &message)?;
        let envelope = Self {
            protocol: XLMP_PROTOCOL.to_owned(),
            version: XLMP_MAJOR_VERSION,
            message_id,
            correlation_id,
            sender,
            sent_at,
            message,
            signature,
        };
        envelope.validate_integrity()?;
        Ok(envelope)
    }

    pub fn kind(&self) -> MessageKind {
        self.message.kind()
    }

    pub fn expected_message_id(&self) -> Result<MessageId, IdError> {
        derive_message_id(
            &self.correlation_id,
            &self.sender,
            &self.sent_at,
            &self.message,
        )
    }

    /// Canonical, domain-separated bytes that an XLMP signature profile signs.
    /// Cryptographic algorithm and key resolution remain deployment policy.
    pub fn signing_bytes(&self) -> Result<Vec<u8>, CanonicalizationError> {
        canonical_json_bytes(&SigningMaterial {
            domain: XLMP_SIGNATURE_DOMAIN,
            message_id: &self.message_id,
            identity: MessageIdentity {
                protocol: &self.protocol,
                version: self.version,
                correlation_id: &self.correlation_id,
                sender: &self.sender,
                sent_at: &self.sent_at,
                message: &self.message,
            },
        })
    }

    pub fn validate_integrity(&self) -> Result<(), XlmpError> {
        if self.protocol != XLMP_PROTOCOL {
            return Err(XlmpError::UnsupportedProtocol(self.protocol.clone()));
        }
        if self.version != XLMP_MAJOR_VERSION {
            return Err(XlmpError::UnsupportedVersion(self.version));
        }
        if self.sender.trim().is_empty() {
            return Err(XlmpError::EmptySender);
        }
        if self.signature.trim().is_empty() {
            return Err(XlmpError::EmptySignature);
        }
        self.message_id.validate()?;
        if let Some(correlation_id) = &self.correlation_id {
            correlation_id.validate()?;
        }
        self.message.validate_ids()?;
        match &self.message {
            XlmpMessage::Claim(message)
                if message.claim_id != message.claim.derive_claim_id()? =>
            {
                return Err(XlmpError::ClaimIdMismatch);
            }
            XlmpMessage::ProofCandidate(message)
                if message.proof_id != message.proof.derive_proof_id()? =>
            {
                return Err(XlmpError::ProofIdMismatch);
            }
            _ => {}
        }
        let expected = self.expected_message_id()?;
        if self.message_id != expected {
            return Err(XlmpError::MessageIdMismatch);
        }
        Ok(())
    }
}

fn derive_message_id(
    correlation_id: &Option<MessageId>,
    sender: &str,
    sent_at: &DateTime<Utc>,
    message: &XlmpMessage,
) -> Result<MessageId, IdError> {
    MessageId::derive(&MessageIdentity {
        protocol: XLMP_PROTOCOL,
        version: XLMP_MAJOR_VERSION,
        correlation_id,
        sender,
        sent_at,
        message,
    })
}

pub fn verify_observation_commit_reveal(
    committed: &ObservationCommitMessage,
    revealed: &ObservationReceipt,
) -> Result<(), XlmpError> {
    let same_binding = committed.job_id == revealed.job_id
        && committed.receipt_id == revealed.receipt_id
        && committed.node_id == revealed.node_id
        && committed.operator_cluster_id == revealed.operator_cluster_id
        && committed.commitment == revealed.commitment
        && committed.committed_at == revealed.committed_at
        && revealed.revealed_at >= committed.committed_at
        && !revealed.reveal_salt.is_empty()
        && xlemma_core::verify_observation_reveal(revealed, revealed.reveal_salt.as_bytes());
    same_binding
        .then_some(())
        .ok_or(XlmpError::ObservationCommitMismatch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xlemma_core::TheoryId;

    fn claim_message() -> XlmpMessage {
        let claim = ClaimManifest {
            protocol_version: "XLMP/1".into(),
            theory_id: TheoryId::derive(&"theory").unwrap(),
            canonical_elaborated_type: "forall p : Prop, p -> p".into(),
            declaration_name: "XLemma.identity".into(),
            source_artifact: None,
            created_at: Utc::now(),
        };
        XlmpMessage::Claim(ClaimMessage {
            claim_id: claim.derive_claim_id().unwrap(),
            claim,
            contribution_manifest_hash: "blake3:contributions".into(),
            rights_manifest_hash: "blake3:rights".into(),
        })
    }

    #[test]
    fn canonical_envelope_round_trips() {
        let envelope = XlmpEnvelope::new(
            None,
            "did:key:researcher",
            Utc::now(),
            claim_message(),
            "test-signature",
        )
        .unwrap();
        let encoded = serde_json::to_vec(&envelope).unwrap();
        let decoded: XlmpEnvelope = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.kind(), MessageKind::Claim);
        decoded.validate_integrity().unwrap();
    }

    #[test]
    fn mutation_invalidates_message_identifier() {
        let mut envelope = XlmpEnvelope::new(
            None,
            "did:key:researcher",
            Utc::now(),
            claim_message(),
            "test-signature",
        )
        .unwrap();
        let original_signing_bytes = envelope.signing_bytes().unwrap();
        envelope.sender = "did:key:attacker".into();
        assert_ne!(original_signing_bytes, envelope.signing_bytes().unwrap());
        assert!(matches!(
            envelope.validate_integrity(),
            Err(XlmpError::MessageIdMismatch)
        ));
    }

    #[test]
    fn source_metadata_cannot_redefine_claim_identity() {
        let XlmpMessage::Claim(original) = claim_message() else {
            unreachable!();
        };
        let mut renamed = original.claim.clone();
        renamed.declaration_name = "Presentation.only".into();
        renamed.created_at += chrono::Duration::days(1);
        assert_eq!(
            original.claim.derive_claim_id().unwrap(),
            renamed.derive_claim_id().unwrap()
        );
    }

    #[test]
    fn envelope_rejects_claim_id_that_does_not_bind_formal_type() {
        let XlmpMessage::Claim(mut message) = claim_message() else {
            unreachable!();
        };
        message.claim.canonical_elaborated_type = "False".into();
        let result = XlmpEnvelope::new(
            None,
            "did:key:researcher",
            Utc::now(),
            XlmpMessage::Claim(message),
            "test-signature",
        );
        assert!(matches!(result, Err(XlmpError::ClaimIdMismatch)));
    }

    #[test]
    fn published_json_vector_has_a_valid_message_identifier() {
        let envelope: XlmpEnvelope = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/xlmp-envelope.json"
        ))
        .unwrap();
        assert_eq!(envelope.message_id, envelope.expected_message_id().unwrap());
        envelope.validate_integrity().unwrap();
    }

    #[test]
    fn required_message_discriminators_are_stable() {
        let kinds = [
            MessageKind::Claim,
            MessageKind::Commit,
            MessageKind::ComputeQuote,
            MessageKind::ProofCandidate,
            MessageKind::VerifyRequest,
            MessageKind::ObservationCommit,
            MessageKind::ObservationReveal,
            MessageKind::Certificate,
            MessageKind::Challenge,
            MessageKind::Finalize,
            MessageKind::Revenue,
            MessageKind::Revalidate,
        ];
        let expected = [
            "XLMP_CLAIM",
            "XLMP_COMMIT",
            "XLMP_COMPUTE_QUOTE",
            "XLMP_PROOF_CANDIDATE",
            "XLMP_VERIFY_REQUEST",
            "XLMP_OBSERVATION_COMMIT",
            "XLMP_OBSERVATION_REVEAL",
            "XLMP_CERTIFICATE",
            "XLMP_CHALLENGE",
            "XLMP_FINALIZE",
            "XLMP_REVENUE",
            "XLMP_REVALIDATE",
        ];
        for (kind, expected) in kinds.into_iter().zip(expected) {
            assert_eq!(serde_json::to_value(kind).unwrap(), expected);
        }
    }
}
