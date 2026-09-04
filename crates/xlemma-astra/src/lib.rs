//! Configurable ASTRA implementation of the XLMP research-prover boundary.
//!
//! ASTRA produces candidates and compute receipts. It is not XLMP, cannot
//! certify its own output, and may be replaced without changing protocol
//! identities, messages, or verification policy.

use async_trait::async_trait;
use chrono::Utc;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, env, time::Instant};
use thiserror::Error;
use xlemma_core::{
    Amount, ArtifactId, AstraComputeReceipt, ComputeReceipt, ComputeService, JobId, ReceiptId,
};
use xlemma_xlmp::{AdapterError, ProverArtifact, ProverRequest, ResearchProver};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AstraConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub reasoning_effort: String,
    pub max_output_tokens: u64,
    pub request_timeout_seconds: u64,
    pub price_input_per_million_minor_units: u128,
    pub price_cached_input_per_million_minor_units: u128,
    pub price_output_per_million_minor_units: u128,
    pub settlement_asset: String,
    pub settlement_decimals: u8,
}

impl AstraConfig {
    pub fn from_env() -> Result<Self, AstraError> {
        Ok(Self {
            api_key: env::var("OPENAI_API_KEY").map_err(|_| AstraError::MissingApiKey)?,
            base_url: env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_owned()),
            model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-6-astra".to_owned()),
            reasoning_effort: env::var("OPENAI_REASONING_EFFORT")
                .unwrap_or_else(|_| "high".to_owned()),
            max_output_tokens: env::var("OPENAI_MAX_OUTPUT_TOKENS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(32_000),
            request_timeout_seconds: 900,
            // Snapshot defaults only. Load production pricing from a dated
            // provider-offer record rather than silently relying on constants.
            price_input_per_million_minor_units: 10_000_000,
            price_cached_input_per_million_minor_units: 1_000_000,
            price_output_per_million_minor_units: 50_000_000,
            settlement_asset: "USDC".to_owned(),
            settlement_decimals: 6,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormalizationRequest {
    pub job_id: JobId,
    pub natural_language_claim: String,
    pub latex_context: Option<String>,
    pub lean_imports: Vec<String>,
    pub namespace: String,
    pub theory_constraints: Vec<String>,
    pub artifact_context_root: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormalClaimCandidate {
    pub declaration_name: String,
    pub lean_statement: String,
    pub assumptions: Vec<String>,
    pub ambiguity_notes: Vec<String>,
    pub output_text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofSearchRequest {
    pub job_id: JobId,
    pub lean_statement: String,
    pub current_lean_file: String,
    pub compiler_diagnostics: Vec<String>,
    pub allowed_imports: Vec<String>,
    pub forbidden_axioms: Vec<String>,
    pub maximum_iterations: u32,
    pub artifact_context_root: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofCandidate {
    pub lean_source: String,
    pub proof_strategy_summary: String,
    pub unresolved_obligations: Vec<String>,
    pub output_text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExplanationRequest {
    pub job_id: JobId,
    pub verified_lean_statement: String,
    pub verified_lean_proof: String,
    pub target_audience: String,
    pub artifact_context_root: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LatexExplanation {
    pub latex: String,
    pub interpretation_warnings: Vec<String>,
    pub output_text: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResponseUsage {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AstraRawResult {
    pub response_id: String,
    pub model: String,
    pub output_text: String,
    pub usage: ResponseUsage,
    pub request_hash: String,
    pub elapsed_ms: u64,
}

#[derive(Debug, Error)]
pub enum AstraError {
    #[error("OPENAI_API_KEY is not configured")]
    MissingApiKey,
    #[error("OpenAI API request failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("OpenAI API returned status {status}: {body}")]
    Api { status: StatusCode, body: String },
    #[error("response did not contain output text")]
    MissingOutput,
    #[error("failed to parse structured ASTRA output: {0}")]
    Parse(#[from] serde_json::Error),
}

#[async_trait]
pub trait AstraProverAdapter: Send + Sync {
    async fn formalize(
        &self,
        request: FormalizationRequest,
    ) -> Result<(FormalClaimCandidate, AstraComputeReceipt), AstraError>;

    async fn search_proof(
        &self,
        request: ProofSearchRequest,
    ) -> Result<(ProofCandidate, AstraComputeReceipt), AstraError>;

    async fn explain(
        &self,
        request: ExplanationRequest,
    ) -> Result<(LatexExplanation, AstraComputeReceipt), AstraError>;
}

#[derive(Clone)]
pub struct OpenAiAstraClient {
    config: AstraConfig,
    http: reqwest::Client,
}

impl OpenAiAstraClient {
    pub fn new(config: AstraConfig) -> Result<Self, AstraError> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(
                config.request_timeout_seconds,
            ))
            .build()?;
        Ok(Self { config, http })
    }

    async fn responses_create(&self, prompt: String) -> Result<AstraRawResult, AstraError> {
        let request_value = json!({
            "model": self.config.model,
            "reasoning": {"effort": self.config.reasoning_effort},
            "max_output_tokens": self.config.max_output_tokens,
            "input": [
                {
                    "role": "developer",
                    "content": [{
                        "type": "input_text",
                        "text": SYSTEM_PROMPT
                    }]
                },
                {
                    "role": "user",
                    "content": [{
                        "type": "input_text",
                        "text": prompt
                    }]
                }
            ]
        });
        let request_bytes = serde_json::to_vec(&request_value)?;
        let request_hash = format!("blake3:{}", blake3::hash(&request_bytes).to_hex());
        let started = Instant::now();

        let response = self
            .http
            .post(format!(
                "{}/responses",
                self.config.base_url.trim_end_matches('/')
            ))
            .bearer_auth(&self.config.api_key)
            .json(&request_value)
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(AstraError::Api { status, body });
        }

        let value: Value = serde_json::from_str(&body)?;
        let output_text = extract_output_text(&value).ok_or(AstraError::MissingOutput)?;
        let usage = extract_usage(&value);
        let response_id = value
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let model = value
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(&self.config.model)
            .to_owned();

        Ok(AstraRawResult {
            response_id,
            model,
            output_text,
            usage,
            request_hash,
            elapsed_ms: started.elapsed().as_millis() as u64,
        })
    }

    fn compute_receipt(
        &self,
        job_id: JobId,
        context_root: String,
        result: &AstraRawResult,
        candidate_roots: Vec<String>,
    ) -> AstraComputeReceipt {
        let charged_units = estimate_charge_minor_units(&self.config, &result.usage);
        let receipt_material = serde_json::json!({
            "job_id": job_id,
            "response_id": result.response_id,
            "request_hash": result.request_hash,
            "context_root": context_root,
            "candidate_roots": candidate_roots,
            "usage": result.usage,
        });
        AstraComputeReceipt {
            receipt_id: ReceiptId::derive(&receipt_material)
                .expect("receipt material is serializable"),
            job_id,
            provider: "openai".to_owned(),
            model_id: result.model.clone(),
            model_snapshot: None,
            reasoning_effort: Some(self.config.reasoning_effort.clone()),
            request_hash: result.request_hash.clone(),
            context_root,
            input_units: result.usage.input_tokens,
            cached_input_units: result.usage.cached_input_tokens,
            output_units: result.usage.output_tokens,
            tool_calls: 0,
            wall_time_ms: result.elapsed_ms,
            retry_count: 0,
            charged_amount: Amount::new(
                charged_units,
                self.config.settlement_asset.clone(),
                self.config.settlement_decimals,
            ),
            candidate_artifact_roots: candidate_roots,
            generated_at: Utc::now(),
            // Production nodes MUST sign through an HSM-backed signer.
            signature: "UNSIGNED_REFERENCE_RECEIPT".to_owned(),
        }
    }
}

#[async_trait]
impl AstraProverAdapter for OpenAiAstraClient {
    async fn formalize(
        &self,
        request: FormalizationRequest,
    ) -> Result<(FormalClaimCandidate, AstraComputeReceipt), AstraError> {
        let prompt = format_formalization_prompt(&request);
        let result = self.responses_create(prompt).await?;
        let parsed: FormalClaimWire = parse_json_object(&result.output_text)?;
        let candidate = FormalClaimCandidate {
            declaration_name: parsed.declaration_name,
            lean_statement: parsed.lean_statement,
            assumptions: parsed.assumptions,
            ambiguity_notes: parsed.ambiguity_notes,
            output_text: result.output_text.clone(),
        };
        let root = hash_text(&candidate.lean_statement);
        let receipt = self.compute_receipt(
            request.job_id,
            request.artifact_context_root,
            &result,
            vec![root],
        );
        Ok((candidate, receipt))
    }

    async fn search_proof(
        &self,
        request: ProofSearchRequest,
    ) -> Result<(ProofCandidate, AstraComputeReceipt), AstraError> {
        let prompt = format_proof_prompt(&request);
        let result = self.responses_create(prompt).await?;
        let parsed: ProofCandidateWire = parse_json_object(&result.output_text)?;
        let candidate = ProofCandidate {
            lean_source: parsed.lean_source,
            proof_strategy_summary: parsed.proof_strategy_summary,
            unresolved_obligations: parsed.unresolved_obligations,
            output_text: result.output_text.clone(),
        };
        let root = hash_text(&candidate.lean_source);
        let receipt = self.compute_receipt(
            request.job_id,
            request.artifact_context_root,
            &result,
            vec![root],
        );
        Ok((candidate, receipt))
    }

    async fn explain(
        &self,
        request: ExplanationRequest,
    ) -> Result<(LatexExplanation, AstraComputeReceipt), AstraError> {
        let prompt = format_explanation_prompt(&request);
        let result = self.responses_create(prompt).await?;
        let parsed: LatexExplanationWire = parse_json_object(&result.output_text)?;
        let explanation = LatexExplanation {
            latex: parsed.latex,
            interpretation_warnings: parsed.interpretation_warnings,
            output_text: result.output_text.clone(),
        };
        let root = hash_text(&explanation.latex);
        let receipt = self.compute_receipt(
            request.job_id,
            request.artifact_context_root,
            &result,
            vec![root],
        );
        Ok((explanation, receipt))
    }
}

impl OpenAiAstraClient {
    async fn run_protocol_proof_task(
        &self,
        request: ProverRequest,
        service: ComputeService,
    ) -> Result<ProverArtifact, AdapterError> {
        let proof_request = ProofSearchRequest {
            job_id: request.job_id,
            lean_statement: required_parameter(&request.parameters, "lean_statement")?.to_owned(),
            current_lean_file: required_parameter(&request.parameters, "current_lean_file")?
                .to_owned(),
            compiler_diagnostics: list_parameter(&request.parameters, "compiler_diagnostics"),
            allowed_imports: list_parameter(&request.parameters, "allowed_imports"),
            forbidden_axioms: list_parameter(&request.parameters, "forbidden_axioms"),
            maximum_iterations: request
                .parameters
                .get("maximum_iterations")
                .map(|value| value.parse::<u32>())
                .transpose()
                .map_err(|error| adapter_error(format!("invalid maximum_iterations: {error}")))?
                .unwrap_or(8),
            artifact_context_root: request.context_root,
        };
        let (candidate, receipt) = AstraProverAdapter::search_proof(self, proof_request)
            .await
            .map_err(|error| adapter_error(error.to_string()))?;
        let root = hash_text(&candidate.lean_source);
        protocol_artifact(receipt, service, root, "text/x-lean")
    }
}

#[async_trait]
impl ResearchProver for OpenAiAstraClient {
    async fn formalize(&self, request: ProverRequest) -> Result<ProverArtifact, AdapterError> {
        let formalization_request = FormalizationRequest {
            job_id: request.job_id,
            natural_language_claim: required_parameter(
                &request.parameters,
                "natural_language_claim",
            )?
            .to_owned(),
            latex_context: request.parameters.get("latex_context").cloned(),
            lean_imports: list_parameter(&request.parameters, "lean_imports"),
            namespace: request
                .parameters
                .get("namespace")
                .cloned()
                .unwrap_or_else(|| "XLemma.Generated".to_owned()),
            theory_constraints: list_parameter(&request.parameters, "theory_constraints"),
            artifact_context_root: request.context_root,
        };
        let (candidate, receipt) = AstraProverAdapter::formalize(self, formalization_request)
            .await
            .map_err(|error| adapter_error(error.to_string()))?;
        let root = hash_text(&candidate.lean_statement);
        protocol_artifact(receipt, ComputeService::Formalization, root, "text/x-lean")
    }

    async fn propose(&self, request: ProverRequest) -> Result<ProverArtifact, AdapterError> {
        self.run_protocol_proof_task(request, ComputeService::ProofSearch)
            .await
    }

    async fn prove(&self, request: ProverRequest) -> Result<ProverArtifact, AdapterError> {
        self.run_protocol_proof_task(request, ComputeService::ProofSearch)
            .await
    }

    async fn repair(&self, request: ProverRequest) -> Result<ProverArtifact, AdapterError> {
        self.run_protocol_proof_task(request, ComputeService::ProofRepair)
            .await
    }

    async fn explain(&self, request: ProverRequest) -> Result<ProverArtifact, AdapterError> {
        let explanation_request = ExplanationRequest {
            job_id: request.job_id,
            verified_lean_statement: required_parameter(
                &request.parameters,
                "verified_lean_statement",
            )?
            .to_owned(),
            verified_lean_proof: required_parameter(&request.parameters, "verified_lean_proof")?
                .to_owned(),
            target_audience: request
                .parameters
                .get("target_audience")
                .cloned()
                .unwrap_or_else(|| "researcher".to_owned()),
            artifact_context_root: request.context_root,
        };
        let (explanation, receipt) = AstraProverAdapter::explain(self, explanation_request)
            .await
            .map_err(|error| adapter_error(error.to_string()))?;
        let root = hash_text(&explanation.latex);
        protocol_artifact(receipt, ComputeService::Explanation, root, "text/x-tex")
    }
}

fn required_parameter<'a>(
    parameters: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, AdapterError> {
    parameters
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| adapter_error(format!("missing required parameter {name}")))
}

fn list_parameter(parameters: &BTreeMap<String, String>, name: &str) -> Vec<String> {
    parameters
        .get(name)
        .map(|value| {
            value
                .lines()
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn protocol_artifact(
    receipt: AstraComputeReceipt,
    service: ComputeService,
    artifact_root: String,
    media_type: &str,
) -> Result<ProverArtifact, AdapterError> {
    let artifact_id = ArtifactId::derive(&(
        "xlmp-prover-artifact-v1",
        &receipt.job_id,
        service,
        &artifact_root,
        media_type,
    ))
    .map_err(|error| adapter_error(error.to_string()))?;
    let metering = BTreeMap::from([
        ("input_units".to_owned(), receipt.input_units),
        ("cached_input_units".to_owned(), receipt.cached_input_units),
        ("output_units".to_owned(), receipt.output_units),
        ("tool_calls".to_owned(), receipt.tool_calls),
        ("wall_time_ms".to_owned(), receipt.wall_time_ms),
        ("retry_count".to_owned(), u64::from(receipt.retry_count)),
    ]);
    let protocol_receipt = ComputeReceipt {
        receipt_id: receipt.receipt_id,
        job_id: receipt.job_id,
        quote_id: None,
        service,
        provider: receipt.provider,
        implementation_id: receipt.model_id,
        implementation_snapshot: receipt.model_snapshot,
        execution_parameters: receipt
            .reasoning_effort
            .map(|effort| BTreeMap::from([("reasoning_effort".to_owned(), effort)]))
            .unwrap_or_default(),
        request_hash: receipt.request_hash,
        context_root: receipt.context_root,
        metering,
        charged_amount: receipt.charged_amount,
        output_artifact_roots: receipt.candidate_artifact_roots,
        completed_at: receipt.generated_at,
        signature: receipt.signature,
    };
    Ok(ProverArtifact {
        artifact_id,
        // A provider-produced candidate cannot assign the final proof identity;
        // that requires the canonical exported proof object.
        proof_id: None,
        media_type: media_type.to_owned(),
        artifact_root,
        compute_receipt: protocol_receipt,
    })
}

fn adapter_error(reason: String) -> AdapterError {
    AdapterError {
        adapter: "astra/openai".to_owned(),
        reason,
    }
}

const SYSTEM_PROMPT: &str = r#"
You are the proof-production component of xLemma. You may propose formal claims,
proof strategies, Lean source, and LaTeX explanations, but you are not a
verifier. Never claim that output is certified. Preserve assumptions exactly,
make ambiguity explicit, do not introduce unapproved axioms, and return only
the requested JSON object. The independent Lean verification network is the
source of formal assurance.
"#;

#[derive(Deserialize)]
struct FormalClaimWire {
    declaration_name: String,
    lean_statement: String,
    #[serde(default)]
    assumptions: Vec<String>,
    #[serde(default)]
    ambiguity_notes: Vec<String>,
}

#[derive(Deserialize)]
struct ProofCandidateWire {
    lean_source: String,
    proof_strategy_summary: String,
    #[serde(default)]
    unresolved_obligations: Vec<String>,
}

#[derive(Deserialize)]
struct LatexExplanationWire {
    latex: String,
    #[serde(default)]
    interpretation_warnings: Vec<String>,
}

fn format_formalization_prompt(request: &FormalizationRequest) -> String {
    format!(
        r#"Formalize the research claim below into a precise Lean 4 declaration.

Natural-language claim:
{claim}

LaTeX context:
{latex}

Allowed imports:
{imports}

Namespace: {namespace}
Theory constraints:
{constraints}

Return JSON exactly as:
{{
  "declaration_name": "Namespace.name",
  "lean_statement": "theorem ... : ... := by",
  "assumptions": ["..."],
  "ambiguity_notes": ["..."]
}}
Do not invent a proof in this step and do not claim verification."#,
        claim = request.natural_language_claim,
        latex = request.latex_context.as_deref().unwrap_or("none"),
        imports = request.lean_imports.join(", "),
        namespace = request.namespace,
        constraints = request.theory_constraints.join("\n"),
    )
}

fn format_proof_prompt(request: &ProofSearchRequest) -> String {
    format!(
        r#"Produce a Lean 4 proof candidate for the exact statement below.

Statement:
{statement}

Current file:
{file}

Lean diagnostics from the previous isolated build:
{diagnostics}

Allowed imports:
{imports}
Forbidden axioms or trust paths:
{forbidden}
Maximum repair iterations for the outer orchestrator: {iterations}

Return JSON exactly as:
{{
  "lean_source": "complete Lean source",
  "proof_strategy_summary": "brief explanation",
  "unresolved_obligations": ["..."]
}}
Do not weaken, strengthen, or silently reinterpret the statement. Do not claim
that the candidate compiles or is verified."#,
        statement = request.lean_statement,
        file = request.current_lean_file,
        diagnostics = request.compiler_diagnostics.join("\n"),
        imports = request.allowed_imports.join(", "),
        forbidden = request.forbidden_axioms.join(", "),
        iterations = request.maximum_iterations,
    )
}

fn format_explanation_prompt(request: &ExplanationRequest) -> String {
    format!(
        r#"Explain this already externally verified Lean theorem and proof in
LaTeX for {audience}. The formal Lean statement remains authoritative. Flag
any place where an informal gloss could overstate the exact theorem.

Verified statement:
{statement}

Verified proof:
{proof}

Return JSON exactly as:
{{
  "latex": "LaTeX body",
  "interpretation_warnings": ["..."]
}}"#,
        audience = request.target_audience,
        statement = request.verified_lean_statement,
        proof = request.verified_lean_proof,
    )
}

fn extract_output_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("output_text").and_then(Value::as_str) {
        return Some(text.to_owned());
    }
    let mut parts = Vec::new();
    for output in value.get("output")?.as_array()? {
        if let Some(content) = output.get("content").and_then(Value::as_array) {
            for item in content {
                if item.get("type").and_then(Value::as_str) == Some("output_text") {
                    if let Some(text) = item.get("text").and_then(Value::as_str) {
                        parts.push(text.to_owned());
                    }
                }
            }
        }
    }
    (!parts.is_empty()).then(|| parts.join("\n"))
}

fn extract_usage(value: &Value) -> ResponseUsage {
    let usage = value.get("usage").unwrap_or(&Value::Null);
    ResponseUsage {
        input_tokens: usage
            .get("input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        cached_input_tokens: usage
            .get("input_tokens_details")
            .and_then(|details| details.get("cached_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        output_tokens: usage
            .get("output_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    }
}

fn parse_json_object<T: for<'de> Deserialize<'de>>(text: &str) -> Result<T, serde_json::Error> {
    if let Ok(parsed) = serde_json::from_str(text) {
        return Ok(parsed);
    }
    match (text.find('{'), text.rfind('}')) {
        (Some(start), Some(end)) if start <= end => serde_json::from_str(&text[start..=end]),
        _ => serde_json::from_str(text),
    }
}

fn hash_text(text: &str) -> String {
    format!("blake3:{}", blake3::hash(text.as_bytes()).to_hex())
}

fn estimate_charge_minor_units(config: &AstraConfig, usage: &ResponseUsage) -> u128 {
    let uncached_input = usage.input_tokens.saturating_sub(usage.cached_input_tokens);
    let input = u128::from(uncached_input)
        .saturating_mul(config.price_input_per_million_minor_units)
        / 1_000_000;
    let cached = u128::from(usage.cached_input_tokens)
        .saturating_mul(config.price_cached_input_per_million_minor_units)
        / 1_000_000;
    let output = u128::from(usage.output_tokens)
        .saturating_mul(config.price_output_per_million_minor_units)
        / 1_000_000;
    input.saturating_add(cached).saturating_add(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_responses_api_output_and_usage() {
        let value = json!({
            "output": [{
                "type": "message",
                "content": [{"type": "output_text", "text": "{\"ok\":true}"}]
            }],
            "usage": {
                "input_tokens": 100,
                "input_tokens_details": {"cached_tokens": 40},
                "output_tokens": 20
            }
        });
        assert_eq!(extract_output_text(&value).unwrap(), "{\"ok\":true}");
        let usage = extract_usage(&value);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.cached_input_tokens, 40);
        assert_eq!(usage.output_tokens, 20);
    }

    #[test]
    fn parser_accepts_json_inside_a_fenced_response() {
        let parsed: FormalClaimWire = parse_json_object(
            "```json\n{\"declaration_name\":\"X.y\",\"lean_statement\":\"theorem y : True := by trivial\"}\n```",
        )
        .unwrap();
        assert_eq!(parsed.declaration_name, "X.y");
    }

    #[test]
    fn charge_distinguishes_cached_and_uncached_input() {
        let config = AstraConfig {
            api_key: "test".into(),
            base_url: "http://127.0.0.1:9999".into(),
            model: "test-model".into(),
            reasoning_effort: "high".into(),
            max_output_tokens: 100,
            request_timeout_seconds: 1,
            price_input_per_million_minor_units: 10_000_000,
            price_cached_input_per_million_minor_units: 1_000_000,
            price_output_per_million_minor_units: 50_000_000,
            settlement_asset: "USDC".into(),
            settlement_decimals: 6,
        };
        let usage = ResponseUsage {
            input_tokens: 1_000_000,
            cached_input_tokens: 500_000,
            output_tokens: 100_000,
        };
        assert_eq!(estimate_charge_minor_units(&config, &usage), 10_500_000);
    }
}
