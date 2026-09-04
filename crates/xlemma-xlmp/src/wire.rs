use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use xlemma_core::{
    canonical_json_bytes, derive_eligible_set_root, ArtifactId, CanonicalizationError,
    CertificateId, Challenge, ClaimId, ClaimManifest, CommitteeSelection,
    CommitteeSortitionRequest, ComputeQuoteReceipt, CredentialRevocation, EligibleNode, IdError,
    JobId, MessageId, NodeBond, NodeCredential, NodeDiscoveryRequest, NodeDiscoveryResult, NodeId,
    NodeReputationSnapshot, NodeServiceAdvertisement, ObservationReceipt, OperatorClusterId,
    OperatorCredential, OperatorId, PoIRCertificate, PolicyId, ProofId, ProofManifest, ReceiptId,
    ResearcherId, RevenueEvent, ServiceMatch, ServiceOrder, UserCredential, VerificationJob,
    VerifiedUserId, XLMP_MAJOR_VERSION, XLMP_PROTOCOL,
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
    #[serde(rename = "XLMP_NODE_ADVERTISE")]
    NodeAdvertise,
    #[serde(rename = "XLMP_DISCOVERY_REQUEST")]
    DiscoveryRequest,
    #[serde(rename = "XLMP_DISCOVERY_RESPONSE")]
    DiscoveryResponse,
    #[serde(rename = "XLMP_SERVICE_ORDER")]
    ServiceOrder,
    #[serde(rename = "XLMP_SERVICE_MATCH")]
    ServiceMatch,
    #[serde(rename = "XLMP_SORTITION_REQUEST")]
    SortitionRequest,
    #[serde(rename = "XLMP_COMMITTEE")]
    Committee,
    #[serde(rename = "XLMP_REPUTATION")]
    Reputation,
    #[serde(rename = "XLMP_BOND")]
    Bond,
    #[serde(rename = "XLMP_USER_CREDENTIAL")]
    UserCredential,
    #[serde(rename = "XLMP_OPERATOR_CREDENTIAL")]
    OperatorCredential,
    #[serde(rename = "XLMP_NODE_CREDENTIAL")]
    NodeCredential,
    #[serde(rename = "XLMP_CREDENTIAL_REVOCATION")]
    CredentialRevocation,
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
    pub verified_user_id: VerifiedUserId,
    pub operator_id: OperatorId,
    pub operator_cluster_id: OperatorClusterId,
    pub user_credential_id: xlemma_core::UserCredentialId,
    pub operator_credential_id: xlemma_core::OperatorCredentialId,
    pub node_credential_id: xlemma_core::NodeCredentialId,
    pub credential_chain_root: String,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAdvertiseMessage {
    pub advertisement: NodeServiceAdvertisement,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryRequestMessage {
    pub request: NodeDiscoveryRequest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryResponseMessage {
    pub result: NodeDiscoveryResult,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceOrderMessage {
    pub order: ServiceOrder,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceMatchMessage {
    pub service_match: ServiceMatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortitionRequestMessage {
    pub request: CommitteeSortitionRequest,
    pub eligible_nodes: Vec<EligibleNode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeMessage {
    pub selection: CommitteeSelection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReputationMessage {
    pub snapshot: NodeReputationSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BondMessage {
    pub bond: NodeBond,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCredentialMessage {
    pub credential: UserCredential,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorCredentialMessage {
    pub credential: OperatorCredential,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCredentialMessage {
    pub credential: NodeCredential,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRevocationMessage {
    pub revocation: CredentialRevocation,
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
    #[serde(rename = "XLMP_NODE_ADVERTISE")]
    NodeAdvertise(NodeAdvertiseMessage),
    #[serde(rename = "XLMP_DISCOVERY_REQUEST")]
    DiscoveryRequest(DiscoveryRequestMessage),
    #[serde(rename = "XLMP_DISCOVERY_RESPONSE")]
    DiscoveryResponse(DiscoveryResponseMessage),
    #[serde(rename = "XLMP_SERVICE_ORDER")]
    ServiceOrder(ServiceOrderMessage),
    #[serde(rename = "XLMP_SERVICE_MATCH")]
    ServiceMatch(ServiceMatchMessage),
    #[serde(rename = "XLMP_SORTITION_REQUEST")]
    SortitionRequest(SortitionRequestMessage),
    #[serde(rename = "XLMP_COMMITTEE")]
    Committee(CommitteeMessage),
    #[serde(rename = "XLMP_REPUTATION")]
    Reputation(ReputationMessage),
    #[serde(rename = "XLMP_BOND")]
    Bond(BondMessage),
    #[serde(rename = "XLMP_USER_CREDENTIAL")]
    UserCredential(UserCredentialMessage),
    #[serde(rename = "XLMP_OPERATOR_CREDENTIAL")]
    OperatorCredential(OperatorCredentialMessage),
    #[serde(rename = "XLMP_NODE_CREDENTIAL")]
    NodeCredential(NodeCredentialMessage),
    #[serde(rename = "XLMP_CREDENTIAL_REVOCATION")]
    CredentialRevocation(CredentialRevocationMessage),
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
            Self::NodeAdvertise(_) => MessageKind::NodeAdvertise,
            Self::DiscoveryRequest(_) => MessageKind::DiscoveryRequest,
            Self::DiscoveryResponse(_) => MessageKind::DiscoveryResponse,
            Self::ServiceOrder(_) => MessageKind::ServiceOrder,
            Self::ServiceMatch(_) => MessageKind::ServiceMatch,
            Self::SortitionRequest(_) => MessageKind::SortitionRequest,
            Self::Committee(_) => MessageKind::Committee,
            Self::Reputation(_) => MessageKind::Reputation,
            Self::Bond(_) => MessageKind::Bond,
            Self::UserCredential(_) => MessageKind::UserCredential,
            Self::OperatorCredential(_) => MessageKind::OperatorCredential,
            Self::NodeCredential(_) => MessageKind::NodeCredential,
            Self::CredentialRevocation(_) => MessageKind::CredentialRevocation,
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
                message.verified_user_id.validate()?;
                message.operator_id.validate()?;
                message.operator_cluster_id.validate()?;
                message.user_credential_id.validate()?;
                message.operator_credential_id.validate()?;
                message.node_credential_id.validate()
            }
            Self::ObservationReveal(message) => {
                message.observation.receipt_id.validate()?;
                message.observation.job_id.validate()?;
                message.observation.node_id.validate()?;
                message.observation.verified_user_id.validate()?;
                message.observation.operator_id.validate()?;
                message.observation.operator_cluster_id.validate()?;
                message.observation.user_credential_id.validate()?;
                message.observation.operator_credential_id.validate()?;
                message.observation.node_credential_id.validate()
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
            Self::NodeAdvertise(message) => {
                message.advertisement.advertisement_id.validate()?;
                message.advertisement.node_id.validate()?;
                message.advertisement.operator_id.validate()?;
                message.advertisement.operator_cluster_id.validate()?;
                message.advertisement.user_credential_id.validate()?;
                message.advertisement.operator_credential_id.validate()?;
                message.advertisement.node_credential_id.validate()?;
                message.advertisement.reputation_snapshot_id.validate()?;
                message.advertisement.bond_id.validate()
            }
            Self::DiscoveryRequest(message) => message.request.discovery_id.validate(),
            Self::DiscoveryResponse(message) => {
                message.result.discovery_id.validate()?;
                for advertisement_id in &message.result.advertisement_ids {
                    advertisement_id.validate()?;
                }
                Ok(())
            }
            Self::ServiceOrder(message) => {
                message.order.order_id.validate()?;
                message.order.job_id.validate()
            }
            Self::ServiceMatch(message) => {
                message.service_match.match_id.validate()?;
                message.service_match.order_id.validate()?;
                message.service_match.advertisement_id.validate()?;
                message.service_match.node_id.validate()?;
                message.service_match.operator_id.validate()?;
                message.service_match.operator_cluster_id.validate()
            }
            Self::SortitionRequest(message) => {
                message.request.sortition_id.validate()?;
                message.request.job_id.validate()?;
                message.request.policy_id.validate()?;
                for node in &message.eligible_nodes {
                    node.node_id.validate()?;
                    node.operator_id.validate()?;
                    node.operator_cluster_id.validate()?;
                    node.credential_chain.user.credential_id.validate()?;
                    node.credential_chain.user.verified_user_id.validate()?;
                    node.credential_chain.operator.credential_id.validate()?;
                    node.credential_chain.operator.operator_id.validate()?;
                    node.credential_chain.node.credential_id.validate()?;
                    node.advertisement_id.validate()?;
                    node.bond_id.validate()?;
                    node.reputation_snapshot_id.validate()?;
                }
                Ok(())
            }
            Self::Committee(message) => {
                message.selection.sortition_id.validate()?;
                message.selection.job_id.validate()?;
                message.selection.policy_id.validate()?;
                for member in &message.selection.members {
                    member.node_id.validate()?;
                    member.verified_user_id.validate()?;
                    member.operator_id.validate()?;
                    member.operator_cluster_id.validate()?;
                    member.user_credential_id.validate()?;
                    member.operator_credential_id.validate()?;
                    member.node_credential_id.validate()?;
                    member.advertisement_id.validate()?;
                    member.bond_id.validate()?;
                    member.reputation_snapshot_id.validate()?;
                }
                Ok(())
            }
            Self::Reputation(message) => {
                message.snapshot.reputation_id.validate()?;
                message.snapshot.node_id.validate()?;
                message.snapshot.operator_id.validate()?;
                message.snapshot.operator_cluster_id.validate()?;
                message.snapshot.policy_id.validate()
            }
            Self::Bond(message) => {
                message.bond.bond_id.validate()?;
                message.bond.node_id.validate()?;
                message.bond.operator_id.validate()?;
                message.bond.operator_cluster_id.validate()?;
                message.bond.slashing_policy_id.validate()
            }
            Self::UserCredential(message) => {
                message.credential.credential_id.validate()?;
                message.credential.verified_user_id.validate()?;
                if let Some(researcher_id) = &message.credential.researcher_id {
                    researcher_id.validate()?;
                }
                Ok(())
            }
            Self::OperatorCredential(message) => {
                message.credential.credential_id.validate()?;
                message.credential.operator_id.validate()?;
                message.credential.verified_user_id.validate()?;
                message.credential.user_credential_id.validate()?;
                message.credential.operator_cluster_id.validate()
            }
            Self::NodeCredential(message) => {
                message.credential.credential_id.validate()?;
                message.credential.node_id.validate()?;
                message.credential.operator_id.validate()?;
                message.credential.operator_credential_id.validate()?;
                message.credential.operator_cluster_id.validate()
            }
            Self::CredentialRevocation(message) => message.revocation.revocation_id.validate(),
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
    #[error("XLMP advertisement identifier does not match its canonical service identity")]
    AdvertisementIdMismatch,
    #[error("XLMP sortition eligible records do not match their committed root")]
    EligibleSetRootMismatch,
    #[error("XLMP service-match identifier does not match its canonical reservation identity")]
    ServiceMatchIdMismatch,
    #[error("XLMP observation reveal does not match its prior commitment")]
    ObservationCommitMismatch,
    #[error("XLMP credential or revocation content is structurally invalid")]
    CredentialIntegrity,
    #[error("XLMP observation lacks its credential-chain binding")]
    ObservationIdentity,
    #[error(transparent)]
    Canonicalization(#[from] CanonicalizationError),
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
            XlmpMessage::UserCredential(message) => message
                .credential
                .validate_integrity()
                .map_err(|_| XlmpError::CredentialIntegrity)?,
            XlmpMessage::OperatorCredential(message) => message
                .credential
                .validate_integrity()
                .map_err(|_| XlmpError::CredentialIntegrity)?,
            XlmpMessage::NodeCredential(message) => message
                .credential
                .validate_integrity()
                .map_err(|_| XlmpError::CredentialIntegrity)?,
            XlmpMessage::CredentialRevocation(message) => {
                message
                    .revocation
                    .validate_integrity()
                    .map_err(|_| XlmpError::CredentialIntegrity)?
            }
            XlmpMessage::SortitionRequest(message) => {
                for node in &message.eligible_nodes {
                    let chain = &node.credential_chain;
                    chain
                        .user
                        .validate_integrity()
                        .map_err(|_| XlmpError::CredentialIntegrity)?;
                    chain
                        .operator
                        .validate_integrity()
                        .map_err(|_| XlmpError::CredentialIntegrity)?;
                    chain
                        .node
                        .validate_integrity()
                        .map_err(|_| XlmpError::CredentialIntegrity)?;
                    chain
                        .status
                        .validate_integrity()
                        .map_err(|_| XlmpError::CredentialIntegrity)?;
                    if node.node_id != chain.node.node_id
                        || node.operator_id != chain.operator.operator_id
                        || node.operator_cluster_id != chain.operator.operator_cluster_id
                        || chain.user.verified_user_id != chain.operator.verified_user_id
                        || chain.user.credential_id != chain.operator.user_credential_id
                        || chain.operator.credential_id != chain.node.operator_credential_id
                    {
                        return Err(XlmpError::CredentialIntegrity);
                    }
                }
            }
            XlmpMessage::ObservationCommit(message)
                if message.credential_chain_root.trim().is_empty() =>
            {
                return Err(XlmpError::ObservationIdentity);
            }
            XlmpMessage::ObservationReveal(message)
                if message.observation.credential_chain_root.trim().is_empty() =>
            {
                return Err(XlmpError::ObservationIdentity);
            }
            _ => {}
        }
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
            XlmpMessage::NodeAdvertise(message)
                if message.advertisement.advertisement_id
                    != message.advertisement.derive_advertisement_id()? =>
            {
                return Err(XlmpError::AdvertisementIdMismatch);
            }
            XlmpMessage::SortitionRequest(message)
                if message.request.eligible_set_root
                    != derive_eligible_set_root(&message.eligible_nodes)? =>
            {
                return Err(XlmpError::EligibleSetRootMismatch);
            }
            XlmpMessage::ServiceMatch(message)
                if message.service_match.match_id != message.service_match.derive_match_id()? =>
            {
                return Err(XlmpError::ServiceMatchIdMismatch);
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
        && committed.verified_user_id == revealed.verified_user_id
        && committed.operator_id == revealed.operator_id
        && committed.operator_cluster_id == revealed.operator_cluster_id
        && committed.user_credential_id == revealed.user_credential_id
        && committed.operator_credential_id == revealed.operator_credential_id
        && committed.node_credential_id == revealed.node_credential_id
        && committed.credential_chain_root == revealed.credential_chain_root
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
    fn published_node_advertisement_has_a_valid_message_identifier() {
        let envelope: XlmpEnvelope = serde_json::from_str(include_str!(
            "../../../examples/node-network/xlmp-node-advertise.json"
        ))
        .unwrap();
        assert_eq!(envelope.message_id, envelope.expected_message_id().unwrap());
        envelope.validate_integrity().unwrap();
    }

    #[test]
    fn user_credential_is_a_native_integrity_checked_message() {
        let credential: UserCredential = serde_json::from_str(include_str!(
            "../../../examples/node-network/user-credential.json"
        ))
        .unwrap();
        let envelope = XlmpEnvelope::new(
            None,
            "did:web:issuer.xlemma.example",
            "2026-09-03T12:00:00Z".parse().unwrap(),
            XlmpMessage::UserCredential(UserCredentialMessage { credential }),
            "example-envelope-signature",
        )
        .unwrap();
        assert_eq!(envelope.kind(), MessageKind::UserCredential);

        let XlmpMessage::UserCredential(mut message) = envelope.message else {
            unreachable!();
        };
        message.credential.public_subject = "did:key:substituted".into();
        assert!(matches!(
            XlmpEnvelope::new(
                None,
                "did:web:issuer.xlemma.example",
                envelope.sent_at,
                XlmpMessage::UserCredential(message),
                "example-envelope-signature",
            ),
            Err(XlmpError::CredentialIntegrity)
        ));
    }

    #[test]
    fn envelope_rejects_advertisement_id_that_does_not_bind_price() {
        let envelope: XlmpEnvelope = serde_json::from_str(include_str!(
            "../../../examples/node-network/xlmp-node-advertise.json"
        ))
        .unwrap();
        let XlmpMessage::NodeAdvertise(mut message) = envelope.message else {
            unreachable!();
        };
        message.advertisement.capabilities[0].price.amount.units += 1;
        let result = XlmpEnvelope::new(
            None,
            "did:key:checker-node-example",
            envelope.sent_at,
            XlmpMessage::NodeAdvertise(message),
            "signature",
        );
        assert!(matches!(result, Err(XlmpError::AdvertisementIdMismatch)));
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
            MessageKind::NodeAdvertise,
            MessageKind::DiscoveryRequest,
            MessageKind::DiscoveryResponse,
            MessageKind::ServiceOrder,
            MessageKind::ServiceMatch,
            MessageKind::SortitionRequest,
            MessageKind::Committee,
            MessageKind::Reputation,
            MessageKind::Bond,
            MessageKind::UserCredential,
            MessageKind::OperatorCredential,
            MessageKind::NodeCredential,
            MessageKind::CredentialRevocation,
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
            "XLMP_NODE_ADVERTISE",
            "XLMP_DISCOVERY_REQUEST",
            "XLMP_DISCOVERY_RESPONSE",
            "XLMP_SERVICE_ORDER",
            "XLMP_SERVICE_MATCH",
            "XLMP_SORTITION_REQUEST",
            "XLMP_COMMITTEE",
            "XLMP_REPUTATION",
            "XLMP_BOND",
            "XLMP_USER_CREDENTIAL",
            "XLMP_OPERATOR_CREDENTIAL",
            "XLMP_NODE_CREDENTIAL",
            "XLMP_CREDENTIAL_REVOCATION",
        ];
        for (kind, expected) in kinds.into_iter().zip(expected) {
            assert_eq!(serde_json::to_value(kind).unwrap(), expected);
        }
    }
}
