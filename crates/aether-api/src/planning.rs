use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use aether_runner::{SortAlgorithm, Task};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod providers;
pub use providers::OllamaModel;
pub(crate) use providers::QWEN_PIN;

#[derive(Debug, Error)]
pub enum PlanningError {
    #[error("planning requires an objective of 1–512 bytes and 1–128 items")]
    InvalidInput,
    #[error("the supervisor must select an explicit, valid model ID")]
    InvalidModel,
    #[error("the model provider slot is busy")]
    Busy,
    #[error("available RAM or VRAM is below the laboratory admission threshold")]
    InsufficientMemory,
    #[error("the model provider is unavailable")]
    Unavailable,
    #[error("the model provider exceeded its deadline")]
    Timeout,
    #[error("the provider refused the request or the model lacks a required capability")]
    Unsupported,
    #[error("the model provider rejected authentication")]
    Authentication,
    #[error("the model provider rate limit was reached")]
    RateLimited,
    #[error("planning was cancelled")]
    Cancelled,
    #[error("the model provider name or its local configuration is invalid")]
    InvalidProvider,
    #[error("hosted planning requires explicit external-transfer and spend approval")]
    ExternalDisabled,
    #[error("the selected hosted model provider has no valid credential")]
    MissingCredential,
    #[error("Ollama must report cloud features disabled to receive a planning request")]
    CloudEnabled,
    #[error("the selected model is absent from the provider's available model list")]
    MissingModel,
    #[error("the model output did not match the bounded sorting proposal format")]
    InvalidResponse,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlanningRequest {
    pub objective: String,
    pub seed: u32,
    pub items: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectiveRequest {
    objective: String,
    seed: u32,
    items: u16,
    #[serde(default)]
    citations: Vec<String>,
}

impl ObjectiveRequest {
    pub fn into_parts(self) -> (PlanningRequest, Vec<String>) {
        (
            PlanningRequest {
                objective: self.objective,
                seed: self.seed,
                items: self.items,
            },
            self.citations,
        )
    }
}

impl PlanningRequest {
    fn validate(&self) -> Result<(), PlanningError> {
        if self.objective.trim().is_empty()
            || self.objective.len() > 512
            || self.objective.chars().any(char::is_control)
            || (Task::Sort {
                seed: self.seed,
                items: self.items,
                algorithm: SortAlgorithm::Standard,
            })
            .validate()
            .is_err()
        {
            return Err(PlanningError::InvalidInput);
        }
        Ok(())
    }

    pub(crate) fn digest(&self) -> Result<String, PlanningError> {
        self.validate()?;
        let encoded = serde_json::to_vec(self).map_err(|_| PlanningError::InvalidInput)?;
        Ok(aether_runner::sha256(&encoded))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Proposal {
    pub algorithm: SortAlgorithm,
    pub items: u16,
    pub seed: u32,
    pub rationale: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelProvenance {
    provider: &'static str,
    requested_model: String,
    reported_model: String,
    model_digest: Option<String>,
    protocol: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TokenUsage {
    input_tokens: u64,
    output_tokens: u64,
}

pub struct Ranking {
    algorithms: Vec<SortAlgorithm>,
    provenance: ModelProvenance,
    tokens: Option<TokenUsage>,
    hypothesis: Option<String>,
    inference: Option<LocalInference>,
}

#[derive(Debug, Serialize)]
struct LocalInference {
    quantization: Option<String>,
    prompt_sha256: String,
    seed: u32,
    context_tokens: u16,
    max_output_tokens: u64,
    temperature: u8,
    thinking: bool,
}

#[derive(Debug, Serialize)]
pub struct PlanningReport {
    schema: &'static str,
    pub(crate) request_sha256: String,
    wall_ms: u128,
    provenance: ModelProvenance,
    tokens: Option<TokenUsage>,
    pub(crate) proposals: Vec<Proposal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hypothesis: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inference: Option<LocalInference>,
}

/// Adapters supply the order of reviewed algorithms. The controller supplies
/// all other settings and the campaign still requires its usual approval.
pub trait ModelAdapter: Send + Sync {
    fn rank(
        &self,
        request: &PlanningRequest,
        cancelled: &AtomicBool,
    ) -> Result<Ranking, PlanningError>;
}

#[derive(Debug, Default)]
pub struct MockModel;

impl ModelAdapter for MockModel {
    fn rank(
        &self,
        request: &PlanningRequest,
        cancelled: &AtomicBool,
    ) -> Result<Ranking, PlanningError> {
        check_cancelled(cancelled)?;
        let candidates = if request.items <= 32 {
            [
                SortAlgorithm::Insertion,
                SortAlgorithm::Standard,
                SortAlgorithm::Selection,
            ]
        } else {
            [
                SortAlgorithm::Standard,
                SortAlgorithm::Insertion,
                SortAlgorithm::Selection,
            ]
        };
        Ok(Ranking {
            algorithms: candidates.to_vec(),
            provenance: ModelProvenance {
                provider: "mock",
                requested_model: "deterministic-v1".into(),
                reported_model: "deterministic-v1".into(),
                model_digest: None,
                protocol: "aether.mock/1",
            },
            tokens: Some(TokenUsage {
                input_tokens: 0,
                output_tokens: 0,
            }),
            hypothesis: None,
            inference: None,
        })
    }
}

pub fn propose(
    adapter: &(impl ModelAdapter + ?Sized),
    request: &PlanningRequest,
) -> Result<Vec<Proposal>, PlanningError> {
    Ok(propose_recorded(adapter, request, &AtomicBool::new(false))?.proposals)
}

pub fn propose_with_provider(
    provider: &str,
    model: &str,
    request: &PlanningRequest,
    cancelled: &AtomicBool,
) -> Result<PlanningReport, PlanningError> {
    let adapter = providers::configured(provider, model)?;
    propose_recorded(adapter.as_ref(), request, cancelled)
}

fn propose_recorded(
    adapter: &(impl ModelAdapter + ?Sized),
    request: &PlanningRequest,
    cancelled: &AtomicBool,
) -> Result<PlanningReport, PlanningError> {
    let request_sha256 = request.digest()?;
    check_cancelled(cancelled)?;
    let started = Instant::now();
    let ranked = adapter.rank(request, cancelled)?;
    check_cancelled(cancelled)?;
    if !(1..=3).contains(&ranked.algorithms.len()) {
        return Err(PlanningError::InvalidResponse);
    }
    let mut proposals: Vec<Proposal> = Vec::with_capacity(ranked.algorithms.len());
    for algorithm in ranked.algorithms {
        if proposals.iter().any(|item| item.algorithm == algorithm) {
            return Err(PlanningError::InvalidResponse);
        }
        let rationale = match algorithm {
            SortAlgorithm::Insertion => {
                "Measure how insertion shifts behave on bounded generated input."
            }
            SortAlgorithm::Selection => {
                "Compare a simple fixed-pass selection algorithm on the same input."
            }
            SortAlgorithm::Standard => {
                "Evaluate the standard-library sort with the same independent reference."
            }
        };
        proposals.push(Proposal {
            algorithm,
            items: request.items,
            seed: request.seed,
            rationale,
        });
    }
    Ok(PlanningReport {
        schema: "aether.planning/1",
        request_sha256,
        wall_ms: started.elapsed().as_millis(),
        provenance: ranked.provenance,
        tokens: ranked.tokens,
        proposals,
        hypothesis: ranked.hypothesis,
        inference: ranked.inference,
    })
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), PlanningError> {
    if cancelled.load(Ordering::Acquire) {
        return Err(PlanningError::Cancelled);
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rank {
    algorithms: Vec<SortAlgorithm>,
    hypothesis: Option<String>,
}

fn ranking_from_json(value: &str) -> Result<Rank, PlanningError> {
    if value.len() > 1024 {
        return Err(PlanningError::InvalidResponse);
    }
    let result: Rank = serde_json::from_str(value).map_err(|_| PlanningError::InvalidResponse)?;
    if result.hypothesis.as_ref().is_some_and(|text| {
        text.trim().is_empty() || text.len() > 240 || text.chars().any(char::is_control)
    }) {
        return Err(PlanningError::InvalidResponse);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_planner_is_deterministic_and_uses_reviewed_bounds() {
        let mut request = PlanningRequest {
            objective: "Compare times on seeded inputs".into(),
            items: 128,
            seed: 11,
        };
        let expected = propose(&MockModel, &request).unwrap();
        assert_eq!(expected, propose(&MockModel, &request).unwrap());
        assert_eq!(expected.len(), 3);
        assert_eq!(expected[0].algorithm, SortAlgorithm::Standard);
        let cancel = AtomicBool::new(false);
        let report = propose_with_provider("mock", "deterministic-v1", &request, &cancel).unwrap();
        assert_eq!(report.proposals, expected);
        assert_eq!(report.provenance.requested_model, "deterministic-v1");
        assert_eq!(report.request_sha256.len(), 64);
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            propose_recorded(&MockModel, &request, &cancel),
            Err(PlanningError::Cancelled)
        ));
        request.items = 129;
        assert!(matches!(
            propose(&MockModel, &request),
            Err(PlanningError::InvalidInput)
        ));
    }

    #[test]
    fn model_output_cannot_supply_execution_parameters() {
        assert!(ranking_from_json(r#"{"algorithms":["insertion"],"shell":"run"}"#).is_err());
        assert!(ranking_from_json(r#"{"algorithms":["download"]}"#).is_err());
        assert!(ranking_from_json(r#"{"algorithms":["standard"],"hypothesis":"\n"}"#).is_err());
        let rank = ranking_from_json(
            r#"{"algorithms":["standard"],"hypothesis":"The output will match the reference."}"#,
        )
        .unwrap();
        assert_eq!(
            rank.hypothesis.as_deref(),
            Some("The output will match the reference.")
        );
        assert!(OllamaModel::new("name:cloud").is_err());
        assert!(OllamaModel::new("other.example/remote:model").is_err());
    }
}
