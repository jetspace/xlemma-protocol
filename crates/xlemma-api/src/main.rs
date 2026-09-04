use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use xlemma_consensus::{evaluate_formal_consensus, FormalConsensusOutcome, FormalConsensusPolicy};
use xlemma_core::{
    ArtifactId, ClaimId, ComputeQuoteId, JobId, ObservationReceipt, PolicyId, ResearcherId,
    TheoryId, VerificationState,
};
use xlemma_x402::{
    insert_payment_required, with_xlemma_extension, PaymentRequired, PaymentRequirement,
    PaymentScheme, ResourceDescription, XLemmaPaymentExtension,
};

#[derive(Clone, Default)]
struct AppState {
    jobs: Arc<RwLock<BTreeMap<String, VerificationJobRecord>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VerificationJobRecord {
    job_id: JobId,
    researcher_id: ResearcherId,
    claim_id: ClaimId,
    theory_id: TheoryId,
    artifact_id: ArtifactId,
    policy_id: PolicyId,
    maximum_budget_minor_units: u128,
    settlement_asset: String,
    state: VerificationState,
    observations: Vec<ObservationReceipt>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateVerificationJob {
    researcher_id: ResearcherId,
    claim_id: ClaimId,
    theory_id: TheoryId,
    artifact_id: ArtifactId,
    policy_id: PolicyId,
    maximum_budget_minor_units: u128,
    settlement_asset: String,
}


#[derive(Debug, Deserialize)]
struct PaymentOfferRequest {
    compute_quote_id: ComputeQuoteId,
    amount: String,
    asset: String,
    network: String,
    pay_to: String,
    max_timeout_seconds: u64,
    artifact_commitment: String,
    model_policy: String,
    rights_manifest_hash: String,
    revenue_route_hash: String,
    delivery_mode: String,
    valid_until: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    protocol: &'static str,
    time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct EvaluateRequest {
    policy: FormalConsensusPolicy,
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
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "resource not found".to_owned()),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::Invalid(message) => (StatusCode::BAD_REQUEST, message),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "info,xlemma_api=debug,tower_http=info".into()
        }))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let state = AppState::default();
    let app = Router::new()
        .route("/health", get(health))
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
        .route(
            "/v1/verification-jobs/{job_id}/payment-required",
            post(create_payment_required),
        )
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

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        protocol: "xlemma/0.2",
        time: Utc::now(),
    })
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
    let job_material = serde_json::json!({
        "researcher_id": request.researcher_id,
        "claim_id": request.claim_id,
        "theory_id": request.theory_id,
        "artifact_id": request.artifact_id,
        "policy_id": request.policy_id,
        "created_at": Utc::now(),
        "nonce": uuid::Uuid::new_v4(),
    });
    let job_id = JobId::derive(&job_material)
        .map_err(|error| ApiError::Invalid(error.to_string()))?;
    let now = Utc::now();
    let record = VerificationJobRecord {
        job_id: job_id.clone(),
        researcher_id: request.researcher_id,
        claim_id: request.claim_id,
        theory_id: request.theory_id,
        artifact_id: request.artifact_id,
        policy_id: request.policy_id,
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
    Json(observation): Json<ObservationReceipt>,
) -> Result<Json<VerificationJobRecord>, ApiError> {
    if observation.job_id.as_str() != job_id {
        return Err(ApiError::Invalid(
            "observation job_id does not match route".to_owned(),
        ));
    }
    let mut jobs = state.jobs.write().await;
    let job = jobs.get_mut(&job_id).ok_or(ApiError::NotFound)?;
    if job
        .observations
        .iter()
        .any(|existing| existing.node_id == observation.node_id)
    {
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
    Json(request): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, ApiError> {
    let mut jobs = state.jobs.write().await;
    let job = jobs.get_mut(&job_id).ok_or(ApiError::NotFound)?;
    let outcome = evaluate_formal_consensus(&request.policy, &job.observations)
        .map_err(|error| ApiError::Invalid(error.to_string()))?;

    job.state = match outcome.status {
        xlemma_core::FormalStatus::Certified => VerificationState::Passed,
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


async fn create_payment_required(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(request): Json<PaymentOfferRequest>,
) -> Result<Response, ApiError> {
    if request.amount.is_empty()
        || request.asset.is_empty()
        || request.network.is_empty()
        || request.pay_to.is_empty()
        || request.artifact_commitment.is_empty()
        || request.model_policy.is_empty()
        || request.rights_manifest_hash.is_empty()
        || request.revenue_route_hash.is_empty()
        || request.delivery_mode.is_empty()
        || request.max_timeout_seconds == 0
        || request.valid_until <= Utc::now()
    {
        return Err(ApiError::Invalid(
            "payment offer contains an empty, expired, or zero-valued required field".to_owned(),
        ));
    }

    let amount_minor_units = request
        .amount
        .parse::<u128>()
        .map_err(|_| ApiError::Invalid("payment amount must be an unsigned integer string".to_owned()))?;
    if amount_minor_units == 0 {
        return Err(ApiError::Invalid(
            "payment amount must be greater than zero".to_owned(),
        ));
    }

    let job = state
        .jobs
        .read()
        .await
        .get(&job_id)
        .cloned()
        .ok_or(ApiError::NotFound)?;
    if request.asset != job.settlement_asset {
        return Err(ApiError::Invalid(
            "payment asset does not match the verification job settlement asset".to_owned(),
        ));
    }
    if amount_minor_units > job.maximum_budget_minor_units {
        return Err(ApiError::Invalid(
            "payment authorization exceeds the verification job maximum budget".to_owned(),
        ));
    }

    let extension = XLemmaPaymentExtension {
        protocol: "xlemma/0.2".to_owned(),
        job_id: job.job_id.clone(),
        researcher_id: job.researcher_id,
        claim_id: job.claim_id,
        proof_id: None,
        artifact_commitment: request.artifact_commitment,
        compute_quote_id: request.compute_quote_id,
        required_verification_policy: job.policy_id,
        model_policy: request.model_policy,
        rights_manifest_hash: request.rights_manifest_hash,
        revenue_route_hash: request.revenue_route_hash,
        delivery_mode: request.delivery_mode,
        valid_until: request.valid_until,
    };
    let required = PaymentRequired {
        x402_version: 2,
        error: "payment authorization required".to_owned(),
        resource: ResourceDescription {
            url: format!("/v1/verification-jobs/{job_id}"),
            description: "Independent xLemma proof verification".to_owned(),
            mime_type: "application/json".to_owned(),
        },
        accepts: vec![PaymentRequirement {
            scheme: PaymentScheme::Upto,
            network: request.network,
            amount: request.amount,
            asset: request.asset,
            pay_to: request.pay_to,
            max_timeout_seconds: request.max_timeout_seconds,
            extra: BTreeMap::from([(
                "paymentIdentifier".to_owned(),
                serde_json::json!(format!("{}:{}", job.job_id, extension.compute_quote_id)),
            )]),
        }],
        extensions: BTreeMap::new(),
    };
    let required = with_xlemma_extension(required, &extension)
        .map_err(|error| ApiError::Invalid(error.to_string()))?;
    let mut response = (StatusCode::PAYMENT_REQUIRED, Json(required.clone())).into_response();
    insert_payment_required(response.headers_mut(), &required)
        .map_err(|error| ApiError::Invalid(error.to_string()))?;
    Ok(response)
}
