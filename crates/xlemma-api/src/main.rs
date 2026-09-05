mod discovery;
mod event_store;

use anyhow::Context;
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path, State},
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    str::FromStr,
    sync::Arc,
};
use tokio::sync::RwLock;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use xlemma_consensus::{
    evaluate_formal_consensus, validate_formal_consensus_policy, FormalConsensusOutcome,
    FormalConsensusPolicy, MAX_COMMITTEE_SLOTS,
};
use xlemma_core::{
    ArtifactId, AvailabilityReceipt, Challenge, ClaimId, ClaimManifest, ComputeReceipt,
    ContributionManifest, DependencyDividend, JobId, LemmaCapsule, License, ObservationReceipt,
    PoIRCertificate, PolicyId, ProofManifest, PublicationRecord, QuarantineRecord, ResearchCredit,
    ResearchVault, ResearcherId, ResearcherNodeManifest, RevenueEvent, RightsManifest,
    SortitionMember, TheoryId, TheoryManifest, VerificationState,
};
use xlemma_xlmp::{
    validate_ed25519_signer, verify_ed25519_detached, verify_ed25519_signature,
    verify_observation_commit_reveal, verify_reproduction_commit_reveal, ObservationCommitMessage,
    XlmpEnvelope, XlmpMessage, XLMP_VERSION,
};

use event_store::{ApiJournalEvent, EventJournal};

#[derive(Clone)]
struct AppState {
    discovery: Arc<RwLock<xlemma_economics::DiscoveryLedger>>,
    jobs: Arc<RwLock<BTreeMap<String, VerificationJobRecord>>>,
    messages: Arc<RwLock<BTreeMap<String, XlmpEnvelope>>>,
    observation_commits: Arc<RwLock<BTreeMap<String, ObservationCommitMessage>>>,
    projection: Arc<RwLock<xlemma_xlmp::ProtocolProjection>>,
    auth_token: Arc<str>,
    trusted_signers: Arc<BTreeSet<String>>,
    node_signers: Arc<BTreeMap<xlemma_core::NodeId, String>>,
    event_journal: Option<Arc<EventJournal>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VerificationJobRecord {
    job_id: JobId,
    researcher_id: ResearcherId,
    claim_id: ClaimId,
    theory_id: TheoryId,
    artifact_id: ArtifactId,
    artifact_root: String,
    policy_id: PolicyId,
    policy: FormalConsensusPolicy,
    committee_members: Vec<SortitionMember>,
    maximum_budget_minor_units: u64,
    settlement_asset: String,
    state: VerificationState,
    observations: Vec<ObservationReceipt>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateVerificationJob {
    researcher_id: ResearcherId,
    claim_id: ClaimId,
    theory_id: TheoryId,
    artifact_id: ArtifactId,
    artifact_root: String,
    policy: FormalConsensusPolicy,
    committee_members: Vec<SortitionMember>,
    maximum_budget_minor_units: u64,
    settlement_asset: String,
}

/// Rejects fields or aliases that the typed XLMP model would otherwise drop
/// during deserialization. This prevents different implementations from
/// authenticating different meanings for the same JSON input.
#[derive(Debug)]
struct StrictXlmpEnvelope(XlmpEnvelope);

impl<'de> Deserialize<'de> for StrictXlmpEnvelope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = Value::deserialize(deserializer)?;
        let envelope: XlmpEnvelope =
            serde_json::from_value(raw.clone()).map_err(serde::de::Error::custom)?;
        let normalized = serde_json::to_value(&envelope).map_err(serde::de::Error::custom)?;
        if raw != normalized {
            return Err(serde::de::Error::custom(
                "XLMP input contains unknown or non-canonical fields",
            ));
        }
        Ok(Self(envelope))
    }
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    protocol: &'static str,
    time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct EvaluateResponse {
    outcome: FormalConsensusOutcome,
    state: VerificationState,
}

#[derive(Debug)]
enum ApiError {
    NotFound,
    Conflict(String),
    Invalid(String),
    Unauthorized,
    Unavailable,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "resource not found".to_owned()),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::Invalid(message) => (StatusCode::BAD_REQUEST, message),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "authentication required".to_owned(),
            ),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "durable protocol state is unavailable".to_owned(),
            ),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

impl AppState {
    fn from_env() -> anyhow::Result<Self> {
        let auth_token =
            std::env::var("XLEMMA_API_AUTH_TOKEN").context("XLEMMA_API_AUTH_TOKEN is required")?;
        if auth_token.len() < 32 {
            anyhow::bail!("XLEMMA_API_AUTH_TOKEN must contain at least 32 bytes");
        }
        let trusted_signers = std::env::var("XLEMMA_TRUSTED_SIGNERS")
            .context("XLEMMA_TRUSTED_SIGNERS is required")?
            .split(',')
            .map(str::trim)
            .filter(|signer| !signer.is_empty())
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();
        if trusted_signers.is_empty() {
            anyhow::bail!("XLEMMA_TRUSTED_SIGNERS must contain at least one signer");
        }
        for signer in &trusted_signers {
            validate_ed25519_signer(signer)
                .context("XLEMMA_TRUSTED_SIGNERS contains an invalid Ed25519 signer")?;
        }
        let raw_node_signers = std::env::var("XLEMMA_TRUSTED_NODE_SIGNERS")
            .context("XLEMMA_TRUSTED_NODE_SIGNERS is required")?;
        let parsed_node_signers: BTreeMap<String, String> = serde_json::from_str(&raw_node_signers)
            .context("XLEMMA_TRUSTED_NODE_SIGNERS must be a JSON object")?;
        let node_signers = parsed_node_signers
            .into_iter()
            .map(|(node_id, signer)| {
                let node_id = xlemma_core::NodeId::from_str(&node_id)
                    .context("invalid NodeID in XLEMMA_TRUSTED_NODE_SIGNERS")?;
                if !trusted_signers.contains(&signer) {
                    anyhow::bail!(
                        "every trusted node signer must also be in XLEMMA_TRUSTED_SIGNERS"
                    );
                }
                Ok((node_id, signer))
            })
            .collect::<anyhow::Result<BTreeMap<_, _>>>()?;
        if node_signers.is_empty() {
            anyhow::bail!("XLEMMA_TRUSTED_NODE_SIGNERS must contain at least one node");
        }
        if node_signers.values().collect::<BTreeSet<_>>().len() != node_signers.len() {
            anyhow::bail!("each trusted NodeID must have a distinct Ed25519 signer");
        }
        let event_log_path =
            std::env::var("XLEMMA_EVENT_LOG_PATH").context("XLEMMA_EVENT_LOG_PATH is required")?;
        let discovery_trust = std::env::var("XLEMMA_DISCOVERY_TRUST_PATH")
            .ok()
            .map(|path| -> anyhow::Result<xlemma_economics::DiscoveryTrust> {
                let trust: xlemma_economics::DiscoveryTrust =
                    serde_json::from_slice(&std::fs::read(path)?)?;
                trust.validate()?;
                Ok(trust)
            })
            .transpose()?;
        let (event_journal, recovered) =
            EventJournal::open_with_discovery(event_log_path, discovery_trust)
                .context("failed to open or authenticate XLEMMA_EVENT_LOG_PATH")?;
        tracing::info!(
            path = %event_journal.path().display(),
            recovered_jobs = recovered.jobs.len(),
            recovered_messages = recovered.messages.len(),
            "recovered durable XLMP protocol state"
        );
        Ok(Self {
            discovery: Arc::new(RwLock::new(recovered.discovery)),
            jobs: Arc::new(RwLock::new(recovered.jobs)),
            messages: Arc::new(RwLock::new(recovered.messages)),
            observation_commits: Arc::new(RwLock::new(recovered.observation_commits)),
            projection: Arc::new(RwLock::new(recovered.projection)),
            auth_token: auth_token.into(),
            trusted_signers: Arc::new(trusted_signers),
            node_signers: Arc::new(node_signers),
            event_journal: Some(Arc::new(event_journal)),
        })
    }

    #[cfg(test)]
    fn for_test(trusted_signer: String) -> Self {
        Self {
            jobs: Arc::default(),
            messages: Arc::default(),
            observation_commits: Arc::default(),
            projection: Arc::default(),
            discovery: Arc::default(),
            auth_token: "test-auth-token-that-is-at-least-32-bytes".into(),
            trusted_signers: Arc::new(BTreeSet::from([trusted_signer])),
            node_signers: Arc::default(),
            event_journal: None,
        }
    }

    fn persist(&self, event: ApiJournalEvent) -> Result<(), ApiError> {
        if let Some(journal) = &self.event_journal {
            journal.append(event).map_err(|error| {
                tracing::error!(error = %error, "durable XLMP state append failed");
                ApiError::Unavailable
            })?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,xlemma_api=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let state = AppState::from_env()?;
    let protected = Router::new()
        .route("/xlmp/v1/messages", post(accept_xlmp_message))
        .route("/xlmp/v1/messages/{message_id}", get(get_xlmp_message))
        .route("/v1/researchers/{researcher_id}", get(get_researcher))
        .route("/v1/theories/{theory_id}", get(get_theory))
        .route("/v1/claims/{claim_id}", get(get_claim))
        .route(
            "/v1/contributions/{manifest_hash}",
            get(get_contribution_manifest),
        )
        .route("/v1/rights/{manifest_hash}", get(get_rights_manifest))
        .route("/v1/proofs/{proof_id}", get(get_proof))
        .route("/v1/certificates/{certificate_id}", get(get_certificate))
        .route(
            "/v1/compute-receipts/{receipt_id}",
            get(get_compute_receipt),
        )
        .route("/v1/research-credits/{credit_id}", get(get_research_credit))
        .route("/v1/research-vaults/{vault_id}", get(get_research_vault))
        .route("/v1/lemmas/{lemma_id}", get(get_capsule))
        .route("/v1/publications/{publication_id}", get(get_publication))
        .route("/v1/licenses/{license_id}", get(get_license))
        .route("/v1/challenges/{challenge_id}", get(get_challenge))
        .route("/v1/quarantines/{quarantine_id}", get(get_quarantine))
        .route("/v1/revenue-events/{revenue_event_id}", get(get_revenue))
        .route("/v1/dependency-dividends/{dividend_id}", get(get_dividend))
        .route(
            "/v1/artifacts/{artifact_id}/availability",
            get(get_availability),
        )
        .route("/v1/discovery/commands", post(discovery::accept))
        .route(
            "/v1/discovery/rounds/{round_id}/submissions/{submission_id}/evidence",
            get(discovery::evidence),
        )
        .route("/v1/discovery/rounds", get(discovery::overview))
        .route(
            "/v1/discovery/rounds/{round_id}/history",
            get(discovery::history),
        )
        .route(
            "/v1/discovery/rounds/{round_id}/queue",
            get(discovery::queue),
        )
        .route(
            "/v1/discovery/rounds/{round_id}/settlement",
            get(discovery::settlement),
        )
        .route("/v1/verification-jobs", post(create_job))
        .route("/v1/verification-jobs/{job_id}", get(get_job))
        .route(
            "/v1/verification-jobs/{job_id}/observations",
            post(add_observation),
        )
        .route(
            "/v1/verification-jobs/{job_id}/evaluate",
            post(evaluate_job),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_authentication,
        ));
    let app = Router::new()
        .route("/health", get(health))
        .merge(protected)
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(ConcurrencyLimitLayer::new(128))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let address: SocketAddr = std::env::var("XLEMMA_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_owned())
        .parse()
        .context("invalid XLEMMA_BIND")?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "xLemma reference API listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn require_authentication(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let presented = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    if presented
        .is_none_or(|token| !constant_time_equal(token.as_bytes(), state.auth_token.as_bytes()))
    {
        return ApiError::Unauthorized.into_response();
    }
    next.run(request).await
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    let length = left.len().max(right.len());
    for index in 0..length {
        let left_byte = left.get(index).copied().unwrap_or_default();
        let right_byte = right.get(index).copied().unwrap_or_default();
        difference |= usize::from(left_byte ^ right_byte);
    }
    difference == 0
}

fn authenticate_envelope(state: &AppState, envelope: &XlmpEnvelope) -> Result<(), ApiError> {
    if !state.trusted_signers.contains(&envelope.sender) {
        return Err(ApiError::Unauthorized);
    }
    verify_ed25519_signature(envelope).map_err(|error| ApiError::Invalid(error.to_string()))
}

fn authenticate_node_sender(
    state: &AppState,
    node_id: &xlemma_core::NodeId,
    sender: &str,
) -> Result<(), ApiError> {
    if state.node_signers.get(node_id).map(String::as_str) != Some(sender) {
        return Err(ApiError::Unauthorized);
    }
    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        protocol: XLMP_VERSION,
        time: Utc::now(),
    })
}

async fn accept_xlmp_message(
    State(state): State<AppState>,
    Json(StrictXlmpEnvelope(envelope)): Json<StrictXlmpEnvelope>,
) -> Result<(StatusCode, Json<XlmpEnvelope>), ApiError> {
    authenticate_envelope(&state, &envelope)?;
    if matches!(&envelope.message, XlmpMessage::Finalize(value) if value.finalized_at > Utc::now())
        || matches!(&envelope.message, XlmpMessage::Publish(value) if value.publication.published_at > Utc::now())
    {
        return Err(ApiError::Invalid(
            "finalization or publication time is in the future".into(),
        ));
    }
    let mut commits = state.observation_commits.write().await;
    match &envelope.message {
        XlmpMessage::ObservationCommit(commit) => {
            authenticate_node_sender(&state, &commit.node_id, &envelope.sender)?;
            if commits.contains_key(commit.receipt_id.as_str()) {
                return Err(ApiError::Conflict(
                    "observation ReceiptID already has a commitment".to_owned(),
                ));
            }
        }
        XlmpMessage::ObservationReveal(reveal) => {
            let observation = &reveal.observation;
            let jobs = state.jobs.read().await;
            let job = jobs
                .get(observation.job_id.as_str())
                .ok_or(ApiError::NotFound)?;
            validate_job_observation(job, observation)?;
            authenticate_node_sender(&state, &observation.node_id, &envelope.sender)?;
            verify_ed25519_detached(
                &envelope.sender,
                &observation.signature,
                &observation
                    .signing_bytes()
                    .map_err(|error| ApiError::Invalid(error.to_string()))?,
            )
            .map_err(|error| ApiError::Invalid(error.to_string()))?;
            let committed = commits
                .get(observation.receipt_id.as_str())
                .ok_or_else(|| {
                    ApiError::Invalid("observation has no prior XLMP commit".to_owned())
                })?;
            verify_observation_commit_reveal(committed, observation)
                .map_err(|error| ApiError::Invalid(error.to_string()))?;
        }
        XlmpMessage::ReproductionObservation(reveal) => {
            let observation = &reveal.observation;
            authenticate_node_sender(&state, &observation.verifier_node_id, &envelope.sender)?;
            verify_ed25519_detached(
                &envelope.sender,
                &observation.signature,
                &observation
                    .signing_bytes()
                    .map_err(|error| ApiError::Invalid(error.to_string()))?,
            )
            .map_err(|error| ApiError::Invalid(error.to_string()))?;
            let committed = commits
                .get(observation.receipt_id.as_str())
                .ok_or_else(|| {
                    ApiError::Invalid(
                        "reproduction observation has no prior XLMP commit".to_owned(),
                    )
                })?;
            verify_reproduction_commit_reveal(committed, observation, &reveal.job, &reveal.profile)
                .map_err(|error| ApiError::Invalid(error.to_string()))?;
        }
        XlmpMessage::Certificate(certificate_message) => {
            let history = state.messages.read().await;
            let jobs = state.jobs.read().await;
            let job = jobs
                .get(certificate_message.certificate.job_id.as_str())
                .ok_or_else(|| {
                    ApiError::Invalid("certificate requires an accepted verification job".into())
                })?;
            validate_certificate_evidence(
                job,
                &certificate_message.certificate,
                &history,
                Utc::now(),
            )?;
        }
        XlmpMessage::ResearchCertificate(certificate_message) => {
            let history = state.messages.read().await;
            let all_observations_previously_authenticated =
                certificate_message.observations.iter().all(|observation| {
                    history.values().any(|prior| {
                        matches!(
                            &prior.message,
                            XlmpMessage::ReproductionObservation(reveal)
                                if &reveal.observation == observation
                        )
                    })
                });
            if !all_observations_previously_authenticated {
                return Err(ApiError::Invalid(
                    "research certificate embeds an observation not authenticated through XLMP ingress"
                        .to_owned(),
                ));
            }
        }
        _ => {}
    }
    let mut messages = state.messages.write().await;
    let mut projection = state.projection.write().await;
    let mut next_projection = projection.clone();
    next_projection
        .apply(&envelope)
        .map_err(|error| match error {
            xlemma_xlmp::ProjectionError::DuplicateMessage(_)
            | xlemma_xlmp::ProjectionError::DuplicateObject => {
                ApiError::Conflict(error.to_string())
            }
            _ => ApiError::Invalid(error.to_string()),
        })?;
    let message_id = envelope.message_id.to_string();
    if messages.contains_key(&message_id) {
        return Err(ApiError::Conflict(
            "XLMP message identifier already exists".to_owned(),
        ));
    }
    state.persist(ApiJournalEvent::MessageAccepted {
        envelope: envelope.clone(),
    })?;
    *projection = next_projection;
    messages.insert(message_id, envelope.clone());
    if let XlmpMessage::ObservationCommit(commit) = &envelope.message {
        commits.insert(commit.receipt_id.to_string(), commit.clone());
    }
    Ok((StatusCode::ACCEPTED, Json(envelope)))
}

async fn get_xlmp_message(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<XlmpEnvelope>, ApiError> {
    state
        .messages
        .read()
        .await
        .get(&message_id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn get_researcher(
    State(state): State<AppState>,
    Path(researcher_id): Path<String>,
) -> Result<Json<ResearcherNodeManifest>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Researcher(value)
            if value.researcher.researcher_id.as_str() == researcher_id =>
        {
            Some(value.researcher.clone())
        }
        _ => None,
    })
    .await
}

async fn get_theory(
    State(state): State<AppState>,
    Path(theory_id): Path<String>,
) -> Result<Json<TheoryManifest>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Theory(value) if value.theory_id.as_str() == theory_id => {
            Some(value.theory.clone())
        }
        _ => None,
    })
    .await
}

async fn get_claim(
    State(state): State<AppState>,
    Path(claim_id): Path<String>,
) -> Result<Json<ClaimManifest>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Claim(value) if value.claim_id.as_str() == claim_id => {
            Some(value.claim.clone())
        }
        _ => None,
    })
    .await
}

async fn get_contribution_manifest(
    State(state): State<AppState>,
    Path(manifest_hash): Path<String>,
) -> Result<Json<ContributionManifest>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Contribution(value) if value.manifest_hash == manifest_hash => {
            Some(value.manifest.clone())
        }
        _ => None,
    })
    .await
}

async fn get_rights_manifest(
    State(state): State<AppState>,
    Path(manifest_hash): Path<String>,
) -> Result<Json<RightsManifest>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Rights(value) if value.manifest_hash == manifest_hash => {
            Some(value.manifest.clone())
        }
        _ => None,
    })
    .await
}

async fn get_proof(
    State(state): State<AppState>,
    Path(proof_id): Path<String>,
) -> Result<Json<ProofManifest>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::ProofCandidate(value) if value.proof_id.as_str() == proof_id => {
            Some(value.proof.clone())
        }
        _ => None,
    })
    .await
}

async fn get_certificate(
    State(state): State<AppState>,
    Path(certificate_id): Path<String>,
) -> Result<Json<PoIRCertificate>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Certificate(value)
            if value.certificate.certificate_id.as_str() == certificate_id =>
        {
            Some(value.certificate.clone())
        }
        _ => None,
    })
    .await
}

async fn get_compute_receipt(
    State(state): State<AppState>,
    Path(receipt_id): Path<String>,
) -> Result<Json<ComputeReceipt>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::ComputeReceipt(value) if value.receipt.receipt_id.as_str() == receipt_id => {
            Some(value.receipt.clone())
        }
        _ => None,
    })
    .await
}

async fn get_research_credit(
    State(state): State<AppState>,
    Path(credit_id): Path<String>,
) -> Result<Json<ResearchCredit>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::ResearchCredit(value) if value.credit.credit_id.as_str() == credit_id => {
            Some(value.credit.clone())
        }
        _ => None,
    })
    .await
}

async fn get_research_vault(
    State(state): State<AppState>,
    Path(vault_id): Path<String>,
) -> Result<Json<ResearchVault>, ApiError> {
    state
        .messages
        .read()
        .await
        .values()
        .filter_map(|envelope| match &envelope.message {
            XlmpMessage::ResearchVault(value) if value.vault.vault_id.as_str() == vault_id => {
                Some(value.vault.clone())
            }
            _ => None,
        })
        .max_by_key(|vault| vault.observed_at)
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn get_capsule(
    State(state): State<AppState>,
    Path(lemma_id): Path<String>,
) -> Result<Json<LemmaCapsule>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Capsule(value) if value.capsule.lemma_id.as_str() == lemma_id => {
            Some(value.capsule.clone())
        }
        _ => None,
    })
    .await
}

async fn get_publication(
    State(state): State<AppState>,
    Path(publication_id): Path<String>,
) -> Result<Json<PublicationRecord>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Publish(value)
            if value.publication.publication_id.as_str() == publication_id =>
        {
            Some(value.publication.clone())
        }
        _ => None,
    })
    .await
}

async fn get_license(
    State(state): State<AppState>,
    Path(license_id): Path<String>,
) -> Result<Json<License>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::License(value) if value.license.license_id.as_str() == license_id => {
            Some(value.license.clone())
        }
        _ => None,
    })
    .await
}

async fn get_challenge(
    State(state): State<AppState>,
    Path(challenge_id): Path<String>,
) -> Result<Json<Challenge>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Challenge(value) if value.challenge.challenge_id.as_str() == challenge_id => {
            Some(value.challenge.clone())
        }
        _ => None,
    })
    .await
}

async fn get_quarantine(
    State(state): State<AppState>,
    Path(quarantine_id): Path<String>,
) -> Result<Json<QuarantineRecord>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Quarantine(value) if value.record.quarantine_id.as_str() == quarantine_id => {
            Some(value.record.clone())
        }
        _ => None,
    })
    .await
}

async fn get_revenue(
    State(state): State<AppState>,
    Path(revenue_event_id): Path<String>,
) -> Result<Json<RevenueEvent>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::Revenue(value)
            if value.event.revenue_event_id.as_str() == revenue_event_id =>
        {
            Some(value.event.clone())
        }
        _ => None,
    })
    .await
}

async fn get_dividend(
    State(state): State<AppState>,
    Path(dividend_id): Path<String>,
) -> Result<Json<DependencyDividend>, ApiError> {
    find_projected(&state, |message| match message {
        XlmpMessage::DependencyDividend(value)
            if value.dividend.dividend_id.as_str() == dividend_id =>
        {
            Some(value.dividend.clone())
        }
        _ => None,
    })
    .await
}

async fn get_availability(
    State(state): State<AppState>,
    Path(artifact_id): Path<String>,
) -> Result<Json<Vec<AvailabilityReceipt>>, ApiError> {
    let receipts = state
        .messages
        .read()
        .await
        .values()
        .filter_map(|envelope| match &envelope.message {
            XlmpMessage::Availability(value)
                if value.receipt.artifact_id.as_str() == artifact_id =>
            {
                Some(value.receipt.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if receipts.is_empty() {
        return Err(ApiError::NotFound);
    }
    Ok(Json(receipts))
}

async fn find_projected<T>(
    state: &AppState,
    select: impl Fn(&XlmpMessage) -> Option<T>,
) -> Result<Json<T>, ApiError> {
    state
        .messages
        .read()
        .await
        .values()
        .find_map(|envelope| select(&envelope.message))
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn create_job(
    State(state): State<AppState>,
    Json(request): Json<CreateVerificationJob>,
) -> Result<(StatusCode, Json<VerificationJobRecord>), ApiError> {
    if request.maximum_budget_minor_units == 0
        || request.maximum_budget_minor_units > 9_007_199_254_740_991
    {
        return Err(ApiError::Invalid(
            "maximum budget must be greater than zero and JCS-safe".to_owned(),
        ));
    }
    if request.artifact_root.trim().is_empty() || request.settlement_asset.trim().is_empty() {
        return Err(ApiError::Invalid(
            "artifact root and settlement asset are required".to_owned(),
        ));
    }
    validate_formal_consensus_policy(&request.policy)
        .map_err(|error| ApiError::Invalid(error.to_string()))?;
    if request.committee_members.is_empty() || request.committee_members.len() > MAX_COMMITTEE_SLOTS
    {
        return Err(ApiError::Invalid(
            "verification job requires a bounded explicit committee roster".to_owned(),
        ));
    }
    request.researcher_id.validate().map_err(invalid_id)?;
    request.claim_id.validate().map_err(invalid_id)?;
    request.theory_id.validate().map_err(invalid_id)?;
    request.artifact_id.validate().map_err(invalid_id)?;
    let mut committee_nodes = BTreeSet::new();
    let mut committee_users = BTreeSet::new();
    let mut committee_operators = BTreeSet::new();
    let mut committee_clusters = BTreeSet::new();
    let mut committee_slots = BTreeSet::new();
    let mut committee_providers = BTreeSet::new();
    let mut committee_regions = BTreeSet::new();
    for member in &request.committee_members {
        member.node_id.validate().map_err(invalid_id)?;
        member.verified_user_id.validate().map_err(invalid_id)?;
        member.operator_id.validate().map_err(invalid_id)?;
        member.operator_cluster_id.validate().map_err(invalid_id)?;
        member.user_credential_id.validate().map_err(invalid_id)?;
        member
            .operator_credential_id
            .validate()
            .map_err(invalid_id)?;
        member.node_credential_id.validate().map_err(invalid_id)?;
        member.advertisement_id.validate().map_err(invalid_id)?;
        member.bond_id.validate().map_err(invalid_id)?;
        member
            .reputation_snapshot_id
            .validate()
            .map_err(invalid_id)?;
        if !matches!(
            member.role,
            xlemma_core::NodeRole::OfficialKernelChecker
                | xlemma_core::NodeRole::IndependentChecker
        ) || !member.credential_tier.can_participate_in_consensus()
            || member.credential_chain_root.trim().is_empty()
            || member.infrastructure_provider.trim().is_empty()
            || member.region.trim().is_empty()
            || member.rank_hash.trim().is_empty()
            || !committee_nodes.insert(member.node_id.clone())
            || !committee_users.insert(member.verified_user_id.clone())
            || !committee_operators.insert(member.operator_id.clone())
            || !committee_clusters.insert(member.operator_cluster_id.clone())
            || !committee_slots.insert((member.role, member.slot))
            || !state.node_signers.contains_key(&member.node_id)
        {
            return Err(ApiError::Invalid(
                "committee roster contains an unauthorized, duplicate, or non-checker node"
                    .to_owned(),
            ));
        }
        committee_providers.insert(member.infrastructure_provider.as_str());
        committee_regions.insert(member.region.as_str());
    }
    let committee_size = request.committee_members.len();
    let required_checker_count = request
        .policy
        .required_family_counts
        .values()
        .try_fold(0_usize, |total, count| total.checked_add(*count))
        .ok_or_else(|| ApiError::Invalid("checker requirements overflow".to_owned()))?;
    if required_checker_count > committee_size
        || request.policy.minimum_verified_users > committee_size
        || request.policy.minimum_operators > committee_size
        || request.policy.minimum_operator_clusters > committee_size
        || request.policy.minimum_infrastructure_providers > committee_providers.len()
        || request.policy.minimum_regions > committee_regions.len()
    {
        return Err(ApiError::Invalid(
            "committee roster cannot satisfy its formal consensus policy".to_owned(),
        ));
    }
    let policy_id =
        PolicyId::derive(&request.policy).map_err(|error| ApiError::Invalid(error.to_string()))?;
    let job_material = serde_json::json!({
        "researcher_id": request.researcher_id,
        "claim_id": request.claim_id,
        "theory_id": request.theory_id,
        "artifact_id": request.artifact_id,
        "artifact_root": request.artifact_root,
        "policy_id": policy_id,
        "created_at": Utc::now(),
        "nonce": uuid::Uuid::new_v4(),
    });
    let job_id =
        JobId::derive(&job_material).map_err(|error| ApiError::Invalid(error.to_string()))?;
    let now = Utc::now();
    let record = VerificationJobRecord {
        job_id: job_id.clone(),
        researcher_id: request.researcher_id,
        claim_id: request.claim_id,
        theory_id: request.theory_id,
        artifact_id: request.artifact_id,
        artifact_root: request.artifact_root,
        policy_id,
        policy: request.policy,
        committee_members: request.committee_members,
        maximum_budget_minor_units: request.maximum_budget_minor_units,
        settlement_asset: request.settlement_asset,
        state: VerificationState::ClaimCommitted,
        observations: Vec::new(),
        created_at: now,
        updated_at: now,
    };
    let mut jobs = state.jobs.write().await;
    let job_key = job_id.to_string();
    if jobs.contains_key(&job_key) {
        return Err(ApiError::Conflict(
            "generated verification job identifier already exists".to_owned(),
        ));
    }
    state.persist(ApiJournalEvent::VerificationJobCreated {
        job: record.clone(),
    })?;
    jobs.insert(job_key, record.clone());
    Ok((StatusCode::CREATED, Json(record)))
}

fn invalid_id(error: xlemma_core::IdError) -> ApiError {
    ApiError::Invalid(error.to_string())
}

async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<VerificationJobRecord>, ApiError> {
    state
        .jobs
        .read()
        .await
        .get(&job_id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound)
}

fn validate_job_observation(
    job: &VerificationJobRecord,
    observation: &ObservationReceipt,
) -> Result<(), ApiError> {
    if observation.job_id != job.job_id {
        return Err(ApiError::Invalid(
            "observation belongs to another job".into(),
        ));
    }
    let authorized_member = job
        .committee_members
        .iter()
        .find(|member| member.node_id == observation.node_id)
        .ok_or(ApiError::Unauthorized)?;
    if authorized_member.verified_user_id != observation.verified_user_id
        || authorized_member.operator_id != observation.operator_id
        || authorized_member.operator_cluster_id != observation.operator_cluster_id
        || authorized_member.user_credential_id != observation.user_credential_id
        || authorized_member.operator_credential_id != observation.operator_credential_id
        || authorized_member.node_credential_id != observation.node_credential_id
        || authorized_member.credential_chain_root != observation.credential_chain_root
        || authorized_member.infrastructure_provider != observation.infrastructure_provider
        || authorized_member.region != observation.region
    {
        return Err(ApiError::Invalid(
            "observation identity does not match the committed committee member".to_owned(),
        ));
    }
    if observation.artifact_root != job.artifact_root {
        return Err(ApiError::Invalid(
            "observation artifact root does not match the verification job".to_owned(),
        ));
    }
    Ok(())
}

fn validate_certificate_evidence(
    job: &VerificationJobRecord,
    certificate: &PoIRCertificate,
    history: &BTreeMap<String, XlmpEnvelope>,
    now: DateTime<Utc>,
) -> Result<(), ApiError> {
    certificate
        .validate_integrity()
        .map_err(|error| ApiError::Invalid(error.to_string()))?;
    // Include every accepted observation, including dissent omitted by a caller
    // and receipts authenticated through the job-specific observation endpoint.
    let mut observations = BTreeMap::new();
    for observation in &job.observations {
        observations.insert(observation.receipt_id.clone(), observation.clone());
    }
    for envelope in history.values() {
        if let XlmpMessage::ObservationReveal(reveal) = &envelope.message {
            if reveal.observation.job_id == job.job_id {
                observations.insert(
                    reveal.observation.receipt_id.clone(),
                    reveal.observation.clone(),
                );
            }
        }
    }
    let observations: Vec<_> = observations.into_values().collect();
    for observation in &observations {
        validate_job_observation(job, observation)?;
    }
    let outcome = evaluate_formal_consensus(&job.policy, &observations)
        .map_err(|error| ApiError::Invalid(error.to_string()))?;
    let expected_receipts: BTreeSet<_> = observations.iter().map(|r| &r.receipt_id).collect();
    let expected_clusters: BTreeSet<_> = observations
        .iter()
        .map(|r| &r.operator_cluster_id)
        .collect();
    let expected_families: BTreeSet<_> = observations
        .iter()
        .filter_map(|r| r.checker_family)
        .collect();
    let challenge_end = i64::try_from(job.policy.challenge_period_seconds)
        .ok()
        .and_then(chrono::Duration::try_seconds)
        .and_then(|duration| certificate.issued_at.checked_add_signed(duration));
    let proof_matches = history.values().any(|envelope| {
        matches!(&envelope.message, XlmpMessage::ProofCandidate(proof)
            if proof.job_id == job.job_id && proof.proof_id == certificate.proof_id
                && proof.proof.claim_id == job.claim_id && proof.artifact_id == job.artifact_id)
    });
    if outcome.status != xlemma_core::FormalStatus::Reproduced
        || certificate.job_id != job.job_id
        || certificate.theory_id != job.theory_id
        || certificate.claim_id != job.claim_id
        || certificate.artifact_id != job.artifact_id
        || certificate.verification_policy_id != job.policy_id
        || certificate
            .observation_receipt_ids
            .iter()
            .collect::<BTreeSet<_>>()
            != expected_receipts
        || certificate
            .operator_cluster_ids
            .iter()
            .collect::<BTreeSet<_>>()
            != expected_clusters
        || certificate
            .checker_families
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            != expected_families
        || outcome.artifact_root.as_ref() != Some(&certificate.artifact_root)
        || outcome.environment_root.as_ref() != Some(&certificate.environment_root)
        || outcome.dependency_root.as_ref() != Some(&certificate.dependency_root)
        || outcome.axiom_set_root.as_ref() != Some(&certificate.axiom_set_root)
        || certificate.issued_at > now
        || certificate.issued_at < job.created_at
        || observations
            .iter()
            .any(|r| r.revealed_at > certificate.issued_at)
        || challenge_end != Some(certificate.challenge_window_ends_at)
        || !proof_matches
    {
        return Err(ApiError::Invalid(
            "certificate does not match the complete authorized job evidence".into(),
        ));
    }
    Ok(())
}

async fn add_observation(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(StrictXlmpEnvelope(envelope)): Json<StrictXlmpEnvelope>,
) -> Result<Json<VerificationJobRecord>, ApiError> {
    authenticate_envelope(&state, &envelope)?;
    let observation = match &envelope.message {
        XlmpMessage::ObservationReveal(message) => message.observation.clone(),
        _ => {
            return Err(ApiError::Invalid(
                "observation endpoint requires XLMP_OBSERVATION_REVEAL".to_owned(),
            ))
        }
    };
    authenticate_node_sender(&state, &observation.node_id, &envelope.sender)?;
    verify_ed25519_detached(
        &envelope.sender,
        &observation.signature,
        &observation
            .signing_bytes()
            .map_err(|error| ApiError::Invalid(error.to_string()))?,
    )
    .map_err(|error| ApiError::Invalid(error.to_string()))?;
    let committed = state
        .observation_commits
        .read()
        .await
        .get(observation.receipt_id.as_str())
        .cloned()
        .ok_or_else(|| ApiError::Invalid("observation has no prior XLMP commit".to_owned()))?;
    verify_observation_commit_reveal(&committed, &observation)
        .map_err(|error| ApiError::Invalid(error.to_string()))?;
    if observation.job_id.as_str() != job_id {
        return Err(ApiError::Invalid(
            "observation job_id does not match route".to_owned(),
        ));
    }
    let mut jobs = state.jobs.write().await;
    let job = jobs.get_mut(&job_id).ok_or(ApiError::NotFound)?;
    if !matches!(
        job.state,
        VerificationState::ClaimCommitted | VerificationState::CheckersRevealed
    ) {
        return Err(ApiError::Conflict(
            "verification job no longer accepts observations".to_owned(),
        ));
    }
    validate_job_observation(job, &observation)?;
    if job.observations.iter().any(|existing| {
        existing.node_id == observation.node_id
            || existing.receipt_id == observation.receipt_id
            || existing.operator_id == observation.operator_id
            || existing.operator_cluster_id == observation.operator_cluster_id
            || existing.verified_user_id == observation.verified_user_id
    }) {
        return Err(ApiError::Conflict(
            "node already submitted an observation".to_owned(),
        ));
    }
    let previous_updated_at = job.updated_at;
    let mut updated = job.clone();
    updated.observations.push(observation);
    updated.state = VerificationState::CheckersRevealed;
    updated.updated_at = Utc::now();
    state.persist(ApiJournalEvent::VerificationJobUpdated {
        job: updated.clone(),
        expected_previous_updated_at: previous_updated_at,
    })?;
    *job = updated.clone();
    Ok(Json(updated))
}

async fn evaluate_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<EvaluateResponse>, ApiError> {
    let mut jobs = state.jobs.write().await;
    let job = jobs.get_mut(&job_id).ok_or(ApiError::NotFound)?;
    if job.state != VerificationState::CheckersRevealed {
        return Err(ApiError::Conflict(
            "verification job is not ready for consensus evaluation".to_owned(),
        ));
    }
    let outcome = evaluate_formal_consensus(&job.policy, &job.observations)
        .map_err(|error| ApiError::Invalid(error.to_string()))?;

    let previous_updated_at = job.updated_at;
    let mut updated = job.clone();
    updated.state = match outcome.status {
        xlemma_core::FormalStatus::Reproduced => VerificationState::Passed,
        xlemma_core::FormalStatus::Rejected => VerificationState::Failed,
        xlemma_core::FormalStatus::Divergent => VerificationState::Divergent,
        xlemma_core::FormalStatus::Quarantined => VerificationState::Quarantined,
        _ => VerificationState::CheckersRevealed,
    };
    updated.updated_at = Utc::now();
    state.persist(ApiJournalEvent::VerificationJobUpdated {
        job: updated.clone(),
        expected_previous_updated_at: previous_updated_at,
    })?;
    *job = updated.clone();

    Ok(Json(EvaluateResponse {
        outcome,
        state: updated.state,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use ed25519_dalek::{Signer, SigningKey};
    use xlemma_core::{
        ClaimManifest, JobId, NodeCredentialId, NodeId, OperatorClusterId, OperatorCredentialId,
        OperatorId, ReceiptId, ReproductionObservation, UserCredentialId, VerificationJob,
        VerificationProfile, VerifiedUserId,
    };
    use xlemma_xlmp::{
        ClaimMessage, ObservationCommitMessage, ReproductionObservationMessage,
        ResearchCertificateMessage, XlmpMessage,
    };

    fn signer_id(key: &SigningKey) -> String {
        format!(
            "ed25519:{}",
            URL_SAFE_NO_PAD.encode(key.verifying_key().as_bytes())
        )
    }

    fn sign_envelope(key: &SigningKey, envelope: &mut XlmpEnvelope) {
        envelope.signature = format!(
            "ed25519:{}",
            URL_SAFE_NO_PAD.encode(key.sign(&envelope.signing_bytes().unwrap()).to_bytes())
        );
    }

    fn test_theory() -> TheoryManifest {
        serde_json::from_str(include_str!("../../../examples/no-arbitrage/theory.json")).unwrap()
    }

    async fn accept_test_theory(state: &AppState) {
        let key = SigningKey::from_bytes(&[7_u8; 32]);
        let theory = test_theory();
        let mut envelope = XlmpEnvelope::new(
            None,
            signer_id(&key),
            Utc::now(),
            XlmpMessage::Theory(xlemma_xlmp::TheoryMessage {
                theory_id: theory.derive_theory_id().unwrap(),
                theory,
            }),
            "pending-signature",
        )
        .unwrap();
        sign_envelope(&key, &mut envelope);
        let _accepted =
            accept_xlmp_message(State(state.clone()), Json(StrictXlmpEnvelope(envelope)))
                .await
                .unwrap();
    }

    fn claim_envelope() -> (XlmpEnvelope, String) {
        let key = SigningKey::from_bytes(&[7_u8; 32]);
        let sender = signer_id(&key);
        let claim = ClaimManifest {
            protocol_version: XLMP_VERSION.to_owned(),
            theory_id: test_theory().derive_theory_id().unwrap(),
            canonical_elaborated_type: "forall p : Prop, p -> p".into(),
            declaration_name: "XLemma.Api.identity".into(),
            source_artifact: None,
            created_at: Utc::now(),
        };
        let claim_id = claim.derive_claim_id().unwrap();
        let mut envelope = XlmpEnvelope::new(
            None,
            sender.clone(),
            Utc::now(),
            XlmpMessage::Claim(ClaimMessage {
                claim_id,
                claim,
                contribution_manifest_hash: "blake3:contributions".into(),
                rights_manifest_hash: "blake3:rights".into(),
            }),
            "pending-signature",
        )
        .unwrap();
        sign_envelope(&key, &mut envelope);
        (envelope, sender)
    }

    fn published_reproduction() -> (
        VerificationProfile,
        VerificationJob,
        ReproductionObservation,
    ) {
        let profile = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/computational-verification-profile.json"
        ))
        .unwrap();
        let job = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/computational-verification-job.json"
        ))
        .unwrap();
        let observation = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/computational-observation-a.json"
        ))
        .unwrap();
        (profile, job, observation)
    }

    fn node_state(key: &SigningKey, node_id: NodeId) -> AppState {
        let signer = signer_id(key);
        AppState {
            jobs: Arc::default(),
            messages: Arc::default(),
            observation_commits: Arc::default(),
            projection: Arc::default(),
            discovery: Arc::default(),
            auth_token: "test-auth-token-that-is-at-least-32-bytes".into(),
            trusted_signers: Arc::new(BTreeSet::from([signer.clone()])),
            node_signers: Arc::new(BTreeMap::from([(node_id, signer)])),
            event_journal: None,
        }
    }

    fn journal_state(path: &std::path::Path, trusted_signer: String) -> AppState {
        let (journal, recovered) = EventJournal::open(path).unwrap();
        AppState {
            discovery: Arc::new(RwLock::new(recovered.discovery)),
            jobs: Arc::new(RwLock::new(recovered.jobs)),
            messages: Arc::new(RwLock::new(recovered.messages)),
            observation_commits: Arc::new(RwLock::new(recovered.observation_commits)),
            projection: Arc::new(RwLock::new(recovered.projection)),
            auth_token: "test-auth-token-that-is-at-least-32-bytes".into(),
            trusted_signers: Arc::new(BTreeSet::from([trusted_signer])),
            node_signers: Arc::default(),
            event_journal: Some(Arc::new(journal)),
        }
    }

    pub(crate) fn certificate_fixture() -> (
        VerificationJobRecord,
        PoIRCertificate,
        BTreeMap<String, XlmpEnvelope>,
    ) {
        let observations: Vec<ObservationReceipt> = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/observations.json"
        ))
        .unwrap();
        let selection: xlemma_core::CommitteeSelection = serde_json::from_str(include_str!(
            "../../../examples/node-network/committee-selection.json"
        ))
        .unwrap();
        let policy: FormalConsensusPolicy =
            serde_json::from_str(include_str!("../../../examples/no-arbitrage/policy.json"))
                .unwrap();
        let proof: ProofManifest =
            serde_json::from_str(include_str!("../../../examples/no-arbitrage/proof.json"))
                .unwrap();
        let members = observations
            .iter()
            .map(|r| {
                let mut member = selection.members[0].clone();
                member.node_id = r.node_id.clone();
                member.verified_user_id = r.verified_user_id.clone();
                member.operator_id = r.operator_id.clone();
                member.operator_cluster_id = r.operator_cluster_id.clone();
                member.user_credential_id = r.user_credential_id.clone();
                member.operator_credential_id = r.operator_credential_id.clone();
                member.node_credential_id = r.node_credential_id.clone();
                member.credential_chain_root = r.credential_chain_root.clone();
                member.infrastructure_provider = r.infrastructure_provider.clone();
                member.region = r.region.clone();
                member
            })
            .collect();
        let now = Utc::now();
        let job = VerificationJobRecord {
            job_id: observations[0].job_id.clone(),
            researcher_id: ResearcherId::derive(&"certificate-researcher").unwrap(),
            claim_id: proof.claim_id.clone(),
            theory_id: test_theory().derive_theory_id().unwrap(),
            artifact_id: proof.artifact_id.clone(),
            artifact_root: observations[0].artifact_root.clone(),
            policy_id: PolicyId::derive(&policy).unwrap(),
            policy,
            committee_members: members,
            maximum_budget_minor_units: 100,
            settlement_asset: "USDC".into(),
            state: VerificationState::CheckersRevealed,
            created_at: observations[0].committed_at - chrono::Duration::hours(1),
            updated_at: now,
            observations,
        };
        let first = &job.observations[0];
        let mut certificate = PoIRCertificate {
            certificate_id: xlemma_core::CertificateId::derive(&"pending").unwrap(),
            job_id: job.job_id.clone(),
            theory_id: job.theory_id.clone(),
            claim_id: job.claim_id.clone(),
            proof_id: proof.derive_proof_id().unwrap(),
            artifact_id: job.artifact_id.clone(),
            verification_policy_id: job.policy_id.clone(),
            observation_receipt_ids: job
                .observations
                .iter()
                .map(|r| r.receipt_id.clone())
                .collect(),
            checker_families: job
                .observations
                .iter()
                .filter_map(|r| r.checker_family)
                .collect(),
            operator_cluster_ids: job
                .observations
                .iter()
                .map(|r| r.operator_cluster_id.clone())
                .collect(),
            artifact_root: first.artifact_root.clone(),
            environment_root: first.environment_root.clone(),
            dependency_root: first.dependency_root.clone(),
            axiom_set_root: first.axiom_set_root.clone(),
            formal_status: xlemma_core::FormalStatus::Certified,
            assurance_level: xlemma_core::AssuranceLevel::FormallyCertified,
            issued_at: now,
            challenge_window_ends_at: now + chrono::Duration::seconds(86_400),
            aggregate_signature: "test-committee-signature".into(),
        };
        certificate.certificate_id = certificate.derive_certificate_id().unwrap();
        let envelope = XlmpEnvelope::new(
            None,
            "test-sender",
            now,
            XlmpMessage::ProofCandidate(xlemma_xlmp::ProofCandidateMessage {
                job_id: job.job_id.clone(),
                proof_id: certificate.proof_id.clone(),
                artifact_id: proof.artifact_id.clone(),
                proof,
            }),
            "test-signature",
        )
        .unwrap();
        (
            job,
            certificate,
            BTreeMap::from([(envelope.message_id.to_string(), envelope)]),
        )
    }

    #[test]
    fn formal_certificate_requires_complete_policy_bound_evidence() {
        let (job, certificate, history) = certificate_fixture();
        assert!(validate_certificate_evidence(&job, &certificate, &history, Utc::now()).is_ok());
        for mutation in 0..9 {
            let mut changed = certificate.clone();
            match mutation {
                0 => {
                    changed.observation_receipt_ids.pop();
                }
                1 => changed.artifact_root = "blake3:substituted".into(),
                2 => changed.environment_root = "blake3:substituted".into(),
                3 => changed.verification_policy_id = PolicyId::derive(&"weaker").unwrap(),
                4 => {
                    changed.operator_cluster_ids[0] =
                        OperatorClusterId::derive(&"fabricated").unwrap()
                }
                5 => changed.challenge_window_ends_at -= chrono::Duration::seconds(1),
                6 => changed.theory_id = TheoryId::derive(&"other-theory").unwrap(),
                7 => changed.dependency_root = "blake3:substituted".into(),
                _ => changed.axiom_set_root = "blake3:substituted".into(),
            }
            changed.certificate_id = changed.derive_certificate_id().unwrap();
            assert!(
                validate_certificate_evidence(&job, &changed, &history, Utc::now()).is_err(),
                "mutation {mutation}"
            );
        }
        let mut unauthorized = job.clone();
        unauthorized.committee_members[0].operator_id =
            OperatorId::derive(&"other-operator").unwrap();
        assert!(
            validate_certificate_evidence(&unauthorized, &certificate, &history, Utc::now())
                .is_err()
        );
        let mut dissent = job.clone();
        let receipt = &mut dissent.observations[0];
        receipt.verdict = xlemma_core::ObservationVerdict::Fail;
        receipt.observation_root = receipt.expected_observation_root().unwrap();
        receipt.commitment = xlemma_core::observation_commitment(
            &receipt.job_id,
            receipt.verdict,
            &receipt.observation_root,
            receipt.reveal_salt.as_bytes(),
        );
        receipt.receipt_id = receipt.expected_receipt_id().unwrap();
        assert!(receipt.validate_integrity().is_ok());
        assert!(
            validate_certificate_evidence(&dissent, &certificate, &history, Utc::now()).is_err()
        );
    }

    #[test]
    fn certificate_content_changes_require_a_new_identity() {
        let (_, mut certificate, _) = certificate_fixture();
        assert!(certificate.validate_integrity().is_ok());
        certificate.artifact_root = "blake3:altered".into();
        assert!(certificate.validate_integrity().is_err());
    }

    #[tokio::test]
    async fn orphan_claim_is_rejected_without_mutating_api_projection() {
        let (envelope, sender) = claim_envelope();
        let state = AppState::for_test(sender);
        let root = state.projection.read().await.state_root().unwrap();
        assert!(
            accept_xlmp_message(State(state.clone()), Json(StrictXlmpEnvelope(envelope)))
                .await
                .is_err()
        );
        assert!(state.messages.read().await.is_empty());
        assert_eq!(state.projection.read().await.state_root().unwrap(), root);
    }

    #[tokio::test]
    async fn xlmp_ingress_is_append_only_and_retrievable() {
        let (envelope, sender) = claim_envelope();
        let state = AppState::for_test(sender);
        let message_id = envelope.message_id.to_string();
        accept_test_theory(&state).await;

        let accepted = accept_xlmp_message(
            State(state.clone()),
            Json(StrictXlmpEnvelope(envelope.clone())),
        )
        .await
        .unwrap();
        assert_eq!(accepted.0, StatusCode::ACCEPTED);

        let fetched = get_xlmp_message(State(state.clone()), Path(message_id))
            .await
            .unwrap();
        assert_eq!(fetched.0, envelope);
        let XlmpMessage::Claim(claim_message) = &envelope.message else {
            unreachable!();
        };
        let projected = get_claim(
            State(state.clone()),
            Path(claim_message.claim_id.to_string()),
        )
        .await
        .unwrap();
        assert_eq!(projected.0, claim_message.claim);

        let duplicate = accept_xlmp_message(State(state), Json(StrictXlmpEnvelope(envelope))).await;
        assert!(matches!(duplicate, Err(ApiError::Conflict(_))));
    }

    #[tokio::test]
    async fn accepted_xlmp_message_survives_api_restart() {
        let (envelope, sender) = claim_envelope();
        let message_id = envelope.message_id.to_string();
        let path = std::env::temp_dir().join(format!(
            "xlemma-api-restart-{}-{}.jsonl",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let state = journal_state(&path, sender.clone());
        accept_test_theory(&state).await;
        let _accepted = accept_xlmp_message(
            State(state.clone()),
            Json(StrictXlmpEnvelope(envelope.clone())),
        )
        .await
        .unwrap();
        drop(state);

        let restarted = journal_state(&path, sender);
        let recovered = get_xlmp_message(State(restarted.clone()), Path(message_id))
            .await
            .unwrap();
        assert_eq!(recovered.0, envelope);
        drop(restarted);
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn xlmp_ingress_rejects_content_identity_mismatch() {
        let (mut envelope, sender) = claim_envelope();
        if let XlmpMessage::Claim(message) = &mut envelope.message {
            message.rights_manifest_hash = "blake3:mutated".into();
        }
        let result = accept_xlmp_message(
            State(AppState::for_test(sender)),
            Json(StrictXlmpEnvelope(envelope)),
        )
        .await;
        assert!(matches!(result, Err(ApiError::Invalid(_))));
    }

    #[tokio::test]
    async fn xlmp_ingress_rejects_a_noncryptographic_signature() {
        let (mut envelope, sender) = claim_envelope();
        envelope.signature = "test-signature".into();
        let result = accept_xlmp_message(
            State(AppState::for_test(sender)),
            Json(StrictXlmpEnvelope(envelope)),
        )
        .await;
        assert!(matches!(result, Err(ApiError::Invalid(_))));
    }

    #[tokio::test]
    async fn observation_commit_must_be_signed_by_the_registered_node_key() {
        let node_key = SigningKey::from_bytes(&[8_u8; 32]);
        let wrong_key = SigningKey::from_bytes(&[9_u8; 32]);
        let node_signer = signer_id(&node_key);
        let wrong_signer = signer_id(&wrong_key);
        let node_id = NodeId::derive(&"registered-node").unwrap();
        let state = AppState {
            jobs: Arc::default(),
            messages: Arc::default(),
            observation_commits: Arc::default(),
            projection: Arc::default(),
            discovery: Arc::default(),
            auth_token: "test-auth-token-that-is-at-least-32-bytes".into(),
            trusted_signers: Arc::new(BTreeSet::from([node_signer.clone(), wrong_signer.clone()])),
            node_signers: Arc::new(BTreeMap::from([(node_id.clone(), node_signer)])),
            event_journal: None,
        };
        let mut envelope = XlmpEnvelope::new(
            None,
            wrong_signer,
            Utc::now(),
            XlmpMessage::ObservationCommit(ObservationCommitMessage {
                job_id: JobId::derive(&"job").unwrap(),
                receipt_id: ReceiptId::derive(&"receipt").unwrap(),
                node_id,
                verified_user_id: VerifiedUserId::derive(&"user").unwrap(),
                operator_id: OperatorId::derive(&"operator-id").unwrap(),
                operator_cluster_id: OperatorClusterId::derive(&"operator-cluster").unwrap(),
                user_credential_id: UserCredentialId::derive(&"user-credential").unwrap(),
                operator_credential_id: OperatorCredentialId::derive(&"operator-credential")
                    .unwrap(),
                node_credential_id: NodeCredentialId::derive(&"node-credential").unwrap(),
                credential_chain_root: "blake3:credential-chain".into(),
                commitment: "blake3:commitment".into(),
                committed_at: Utc::now(),
                signature: "covered-by-envelope-signature".into(),
            }),
            "pending-signature",
        )
        .unwrap();
        sign_envelope(&wrong_key, &mut envelope);

        let result = accept_xlmp_message(State(state), Json(StrictXlmpEnvelope(envelope))).await;
        assert!(matches!(result, Err(ApiError::Unauthorized)));
    }

    #[tokio::test]
    async fn generalized_reproduction_requires_commit_then_authenticated_reveal() {
        let key = SigningKey::from_bytes(&[10_u8; 32]);
        let sender = signer_id(&key);
        let (profile, job, mut observation) = published_reproduction();
        let state = node_state(&key, observation.verifier_node_id.clone());

        observation.signature = format!(
            "ed25519:{}",
            URL_SAFE_NO_PAD.encode(key.sign(&observation.signing_bytes().unwrap()).to_bytes())
        );
        let reproduction_message =
            XlmpMessage::ReproductionObservation(ReproductionObservationMessage {
                job: job.clone(),
                profile: profile.clone(),
                observation: observation.clone(),
            });
        let mut early_reveal = XlmpEnvelope::new(
            None,
            sender.clone(),
            Utc::now(),
            reproduction_message.clone(),
            "pending-signature",
        )
        .unwrap();
        sign_envelope(&key, &mut early_reveal);
        let result =
            accept_xlmp_message(State(state.clone()), Json(StrictXlmpEnvelope(early_reveal))).await;
        assert!(matches!(result, Err(ApiError::Invalid(_))));

        let commit = ObservationCommitMessage {
            job_id: observation.job_id.clone(),
            receipt_id: observation.receipt_id.clone(),
            node_id: observation.verifier_node_id.clone(),
            verified_user_id: observation.verified_user_id.clone(),
            operator_id: observation.operator_id.clone(),
            operator_cluster_id: observation.operator_cluster_id.clone(),
            user_credential_id: observation.user_credential_id.clone(),
            operator_credential_id: observation.operator_credential_id.clone(),
            node_credential_id: observation.node_credential_id.clone(),
            credential_chain_root: observation.credential_chain_root.clone(),
            commitment: observation.commitment.clone(),
            committed_at: observation.committed_at,
            signature: "covered-by-envelope-signature".into(),
        };
        let mut commit_envelope = XlmpEnvelope::new(
            None,
            sender.clone(),
            observation.committed_at,
            XlmpMessage::ObservationCommit(commit),
            "pending-signature",
        )
        .unwrap();
        sign_envelope(&key, &mut commit_envelope);
        let _accepted_commit = accept_xlmp_message(
            State(state.clone()),
            Json(StrictXlmpEnvelope(commit_envelope)),
        )
        .await
        .unwrap();

        let mut reveal_envelope = XlmpEnvelope::new(
            None,
            sender,
            observation.reproduced_at,
            reproduction_message,
            "pending-signature",
        )
        .unwrap();
        sign_envelope(&key, &mut reveal_envelope);
        assert_eq!(
            accept_xlmp_message(State(state), Json(StrictXlmpEnvelope(reveal_envelope)),)
                .await
                .unwrap()
                .0,
            StatusCode::ACCEPTED
        );
    }

    #[tokio::test]
    async fn research_certificate_rejects_observations_that_bypassed_ingress() {
        let key = SigningKey::from_bytes(&[11_u8; 32]);
        let sender = signer_id(&key);
        let (profile, job, _) = published_reproduction();
        let observations: Vec<ReproductionObservation> = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/computational-observations.json"
        ))
        .unwrap();
        let certificate = serde_json::from_str(include_str!(
            "../../../examples/no-arbitrage/computational-research-certificate.json"
        ))
        .unwrap();
        let mut envelope = XlmpEnvelope::new(
            None,
            sender.clone(),
            Utc::now(),
            XlmpMessage::ResearchCertificate(ResearchCertificateMessage {
                job,
                profile,
                observations,
                certificate,
            }),
            "pending-signature",
        )
        .unwrap();
        sign_envelope(&key, &mut envelope);

        let result = accept_xlmp_message(
            State(AppState::for_test(sender)),
            Json(StrictXlmpEnvelope(envelope)),
        )
        .await;
        assert!(matches!(result, Err(ApiError::Invalid(_))));
    }

    #[test]
    fn xlmp_json_rejects_unknown_nested_fields() {
        let (envelope, _) = claim_envelope();
        let mut raw = serde_json::to_value(envelope).unwrap();
        raw["message"]["payload"]["claim"]["unsigned_extension"] =
            Value::String("must-not-be-dropped".into());

        assert!(serde_json::from_value::<StrictXlmpEnvelope>(raw).is_err());
    }

    #[test]
    fn bearer_token_comparison_checks_length_and_contents() {
        assert!(constant_time_equal(b"same", b"same"));
        assert!(!constant_time_equal(b"same", b"different"));
        assert!(!constant_time_equal(b"same", b"samf"));
    }
}
