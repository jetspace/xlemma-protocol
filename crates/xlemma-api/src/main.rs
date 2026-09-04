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
    ArtifactId, ClaimId, JobId, ObservationReceipt, PolicyId, ResearcherId, SortitionMember,
    TheoryId, VerificationState,
};
use xlemma_xlmp::{
    validate_ed25519_signer, verify_ed25519_detached, verify_ed25519_signature,
    verify_observation_commit_reveal, ObservationCommitMessage, XlmpEnvelope, XlmpMessage,
    XLMP_VERSION,
};

#[derive(Clone)]
struct AppState {
    jobs: Arc<RwLock<BTreeMap<String, VerificationJobRecord>>>,
    messages: Arc<RwLock<BTreeMap<String, XlmpEnvelope>>>,
    observation_commits: Arc<RwLock<BTreeMap<String, ObservationCommitMessage>>>,
    auth_token: Arc<str>,
    trusted_signers: Arc<BTreeSet<String>>,
    node_signers: Arc<BTreeMap<xlemma_core::NodeId, String>>,
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
    maximum_budget_minor_units: u128,
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
    maximum_budget_minor_units: u128,
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
        Ok(Self {
            jobs: Arc::default(),
            messages: Arc::default(),
            observation_commits: Arc::default(),
            auth_token: auth_token.into(),
            trusted_signers: Arc::new(trusted_signers),
            node_signers: Arc::new(node_signers),
        })
    }

    #[cfg(test)]
    fn for_test(trusted_signer: String) -> Self {
        Self {
            jobs: Arc::default(),
            messages: Arc::default(),
            observation_commits: Arc::default(),
            auth_token: "test-auth-token-that-is-at-least-32-bytes".into(),
            trusted_signers: Arc::new(BTreeSet::from([trusted_signer])),
            node_signers: Arc::default(),
        }
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
        _ => {}
    }
    let mut messages = state.messages.write().await;
    match messages.entry(envelope.message_id.to_string()) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(envelope.clone());
        }
        std::collections::btree_map::Entry::Occupied(_) => {
            return Err(ApiError::Conflict(
                "XLMP message identifier already exists".to_owned(),
            ));
        }
    }
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

async fn create_job(
    State(state): State<AppState>,
    Json(request): Json<CreateVerificationJob>,
) -> Result<(StatusCode, Json<VerificationJobRecord>), ApiError> {
    if request.maximum_budget_minor_units == 0 {
        return Err(ApiError::Invalid(
            "maximum budget must be greater than zero".to_owned(),
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
    match jobs.entry(job_id.to_string()) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(record.clone());
        }
        std::collections::btree_map::Entry::Occupied(_) => {
            return Err(ApiError::Conflict(
                "generated verification job identifier already exists".to_owned(),
            ));
        }
    }
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
    job.observations.push(observation);
    job.state = VerificationState::CheckersRevealed;
    job.updated_at = Utc::now();
    Ok(Json(job.clone()))
}

async fn evaluate_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<EvaluateResponse>, ApiError> {
    let mut jobs = state.jobs.write().await;
    let job = jobs.get_mut(&job_id).ok_or(ApiError::NotFound)?;
    let outcome = evaluate_formal_consensus(&job.policy, &job.observations)
        .map_err(|error| ApiError::Invalid(error.to_string()))?;

    job.state = match outcome.status {
        xlemma_core::FormalStatus::Reproduced => VerificationState::Passed,
        xlemma_core::FormalStatus::Rejected => VerificationState::Failed,
        xlemma_core::FormalStatus::Divergent => VerificationState::Divergent,
        xlemma_core::FormalStatus::Quarantined => VerificationState::Quarantined,
        _ => VerificationState::CheckersRevealed,
    };
    job.updated_at = Utc::now();

    Ok(Json(EvaluateResponse {
        outcome,
        state: job.state,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use ed25519_dalek::{Signer, SigningKey};
    use xlemma_core::{
        ClaimManifest, JobId, NodeCredentialId, NodeId, OperatorClusterId, OperatorCredentialId,
        OperatorId, ReceiptId, UserCredentialId, VerifiedUserId,
    };
    use xlemma_xlmp::{ClaimMessage, ObservationCommitMessage, XlmpMessage};

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

    fn claim_envelope() -> (XlmpEnvelope, String) {
        let key = SigningKey::from_bytes(&[7_u8; 32]);
        let sender = signer_id(&key);
        let claim = ClaimManifest {
            protocol_version: XLMP_VERSION.to_owned(),
            theory_id: TheoryId::derive(&"api-test-theory").unwrap(),
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

    #[tokio::test]
    async fn xlmp_ingress_is_append_only_and_retrievable() {
        let (envelope, sender) = claim_envelope();
        let state = AppState::for_test(sender);
        let message_id = envelope.message_id.to_string();

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

        let duplicate = accept_xlmp_message(State(state), Json(StrictXlmpEnvelope(envelope))).await;
        assert!(matches!(duplicate, Err(ApiError::Conflict(_))));
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
            auth_token: "test-auth-token-that-is-at-least-32-bytes".into(),
            trusted_signers: Arc::new(BTreeSet::from([node_signer.clone(), wrong_signer.clone()])),
            node_signers: Arc::new(BTreeMap::from([(node_id.clone(), node_signer)])),
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
