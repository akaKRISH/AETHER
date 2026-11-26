use aether_runner::{
    Output, SortAlgorithm, Task, WORKER_VERSION, evaluate, optimization_input, sha256,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{Candidate, Rejection};
use crate::{
    Result,
    lab::Lab,
    literature::{Citation, TextManifest},
};

const BUNDLE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../research/literature/apo02-reviewed-excerpts.json"
));
const BUNDLE_SHA256: &str = "8502f5a904afd3aab8e19680eb1cdd1fa863adb7d8a9ff05786a5a547322ab0a";
const LICENSE_URL: &str = "https://creativecommons.org/licenses/by/4.0/";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Recipe {
    Sort {
        case: u8,
        items: u16,
    },
    Search {
        case: u8,
        items: u16,
    },
    MaxSubarray {
        case: u8,
        items: u16,
    },
    Grounding {
        source: usize,
        bundle_sha256: String,
    },
    Uncertainty {
        case: u8,
    },
    Abstention {
        case: u8,
    },
    ScientificDesign {
        case: u8,
    },
}

impl Recipe {
    pub(super) fn selects_reviewed_source(&self) -> bool {
        matches!(self, Self::Grounding { source, bundle_sha256 } if *source < 2 && bundle_sha256 == BUNDLE_SHA256)
    }
}

#[derive(Deserialize)]
struct Reviewed {
    citation: Citation,
    source_url: String,
    license: String,
    source_section: String,
    fixture_heading: String,
    quote: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct Evidence {
    pub paper: String,
    pub document: String,
    pub source_sha256: String,
    node: String,
    revision_sha256: String,
    start_byte: usize,
    end_byte: usize,
    title: String,
    authors: Vec<String>,
    source_url: String,
    source_section: String,
    license: String,
    license_url: &'static str,
    source_authorship: &'static str,
    bundle_sha256: &'static str,
}

#[derive(Debug)]
pub(super) struct Grounding {
    quote: String,
    evidence: Evidence,
}

pub(super) fn reviewed_sources(lab: &Lab) -> Result<Vec<Grounding>> {
    if sha256(BUNDLE) != BUNDLE_SHA256 {
        return Err("reviewed literature bundle failed its SHA-256 pin".into());
    }
    let reviewed: Vec<Reviewed> = serde_json::from_slice(BUNDLE)?;
    let mut catalog = lab.catalog()?;
    let mut results = Vec::new();
    for row in reviewed {
        let Some(doi) = row
            .citation
            .doi
            .as_deref()
            .filter(|doi| !doi.trim().is_empty())
        else {
            return Err("reviewed source is missing its DOI".into());
        };
        if row.license != "CC-BY-4.0"
            || row.source_url != format!("https://doi.org/{doi}")
            || !matches!(row.fixture_heading.as_str(), "Claim" | "Limitations")
        {
            return Err("reviewed source failed its rights or identity policy".into());
        }
        let paper = doi.to_owned();
        let heading = format!("## {}\n", row.fixture_heading);
        let span = heading.len()..heading.len() + row.quote.len();
        let bytes = format!("{heading}{}\n", row.quote).into_bytes();
        let expected_sha256 = sha256(&bytes);
        let manifest: TextManifest = serde_json::from_value(json!({
            "paper_id":paper, "source_url":row.source_url, "license":row.license, "rights_confirmed":true
        }))?;
        catalog.import_manual(vec![row.citation.clone()])?;
        let record = catalog.ingest_text_from(manifest, || Ok(bytes))?;
        let document = record["id"]
            .as_str()
            .ok_or("missing archived source identity")?;
        let archive = catalog.source_text(&paper, document)?;
        if archive["record"]["source_sha256"] != expected_sha256
            || archive["record"]["source"]["license"] != row.license
            || archive["text"]
                .as_str()
                .and_then(|text| text.get(span.clone()))
                != Some(row.quote.as_str())
        {
            return Err("reviewed source no longer matches its pinned bytes".into());
        }
        let (node, revision_sha256) = catalog.dataset_passage_ref(
            &paper,
            document,
            span.clone(),
            &row.quote,
            row.fixture_heading == "Limitations",
        )?;
        results.push(Grounding {
            quote: row.quote,
            evidence: Evidence {
                paper,
                document: document.into(),
                source_sha256: expected_sha256,
                node,
                revision_sha256,
                start_byte: span.start,
                end_byte: span.end,
                title: row.citation.title,
                authors: row.citation.authors,
                source_url: row.source_url,
                source_section: row.source_section,
                license: row.license,
                license_url: LICENSE_URL,
                source_authorship: "human_authored",
                bundle_sha256: BUNDLE_SHA256,
            },
        });
    }
    Ok(results)
}

pub(super) fn default_candidates() -> Vec<Value> {
    let mut rows = Vec::new();
    for (kind, first) in [("sort", 0), ("search", 16), ("max_subarray", 32)] {
        for offset in 0..4 {
            rows.push(json!({"recipe":{"kind":kind, "case":first + offset,"items":6 + offset}}));
        }
    }
    for kind in ["uncertainty", "abstention", "scientific_design"] {
        for case in 0..4 {
            rows.push(json!({"recipe":{"kind":kind,"case":case}}));
        }
    }
    for source in 0..2 {
        rows.push(
            json!({"recipe":{"kind":"grounding","source":source,"bundle_sha256":BUNDLE_SHA256}}),
        );
    }
    for variant in [1, 2] {
        rows.push(json!({"recipe":{"kind":"sort","case":0,"items":6},"variant":variant}));
    }
    rows
}

#[derive(Debug)]
pub(super) struct Checked {
    pub input: String,
    pub output: String,
    pub topic: &'static str,
    pub family: String,
    pub case: String,
    pub origin: &'static str,
    pub verification: &'static str,
    pub evidence: Option<Evidence>,
}

fn simulated(recipe: &Recipe) -> Option<(&'static str, &'static str, String, Value)> {
    match *recipe {
        Recipe::Uncertainty { case } => {
            let (low, high) = *[(-5, 7), (-8, -1), (1, 12), (0, 2)].get(case as usize)?;
            let label = if low > 0 {
                "positive"
            } else if high < 0 {
                "negative"
            } else {
                "inconclusive"
            };
            Some((
                "uncertainty",
                "interval_direction",
                format!(
                    "A reported confidence interval for the difference is [{low}, {high}]. Classify its sign as positive, negative, or inconclusive."
                ),
                json!(label),
            ))
        }
        Recipe::Abstention { case } => {
            let (observed, requested) = *[
                ("Only the validated mean is recorded.", "sample count"),
                (
                    "Only marginal counts are reported for two variables.",
                    "joint count",
                ),
                ("Only the model alias is available.", "exact weight hash"),
                (
                    "Only a training receipt is available.",
                    "independent test accuracy",
                ),
            ]
            .get(case as usize)?;
            Some((
                "abstention",
                "missing_information",
                format!("{observed} Give the {requested} or answer insufficient_information."),
                json!("insufficient_information"),
            ))
        }
        Recipe::ScientificDesign { case } => {
            let (same_data, same_budget) =
                *[(true, true), (true, false), (false, true), (false, false)].get(case as usize)?;
            let input = format!(
                "Two algorithms are compared with the same evaluator. Same instances: {same_data}. Same budget: {same_budget}. Is this matched or confounded?"
            );
            Some((
                "scientific_reasoning",
                "matched_controls",
                input,
                json!(if same_data && same_budget {
                    "matched"
                } else {
                    "confounded"
                }),
            ))
        }
        _ => None,
    }
}

pub(super) fn check(candidate: Candidate, seed: u32, sources: &[Grounding]) -> Result<ResultRow> {
    let Candidate {
        recipe,
        variant,
        submission,
    } = candidate;
    if variant > 2 {
        return Ok(Err(Rejection::InvalidCase));
    }
    let case_key = |kind, case| format!("{kind}/{case}");
    let mut evidence = None;
    let (topic, family, case, mut input, ideal, verification) = match &recipe {
        Recipe::Grounding {
            source,
            bundle_sha256,
        } => {
            let Some(reviewed) = sources
                .get(*source)
                .filter(|_| bundle_sha256 == BUNDLE_SHA256)
            else {
                return Ok(Err(Rejection::Provenance));
            };
            let source = &reviewed.evidence;
            let input = format!(
                "Copy the supporting sentence exactly. Source: {} ({}), {} ({}).\n{}",
                source.authors.join(", "),
                source.source_url,
                source.license,
                source.license_url,
                reviewed.quote
            );
            evidence = Some(source.clone());
            (
                "evidence_grounding",
                format!("source/{}", source.paper),
                format!(
                    "passage/{}/{}/{}",
                    source.document, source.start_byte, source.end_byte
                ),
                input,
                json!(reviewed.quote),
                "source_span_checked",
            )
        }
        Recipe::Sort { case, items }
        | Recipe::Search { case, items }
        | Recipe::MaxSubarray { case, items } => {
            if !(4..=16).contains(items) {
                return Ok(Err(Rejection::InvalidCase));
            }
            let seed_hash = sha256(format!("aether.dataset/1/{seed}/{case}").as_bytes());
            let task_seed = u32::from_str_radix(&seed_hash[..8], 16)?;
            let values = optimization_input(task_seed, *items)?;
            let (family, task, input) = match &recipe {
                Recipe::Sort { .. } => (
                    "sort",
                    Task::Sort {
                        seed: task_seed,
                        items: *items,
                        algorithm: SortAlgorithm::Insertion,
                    },
                    format!(
                        "Sort these signed integers in ascending order and return the JSON list, keeping every duplicate. Input values: {values:?}"
                    ),
                ),
                Recipe::Search { .. } => {
                    let target = if case.is_multiple_of(2) {
                        values[0]
                    } else {
                        200
                    };
                    let mut sorted = values.clone();
                    sorted.sort_unstable();
                    (
                        "search",
                        Task::Search {
                            seed: task_seed,
                            items: *items,
                            target,
                        },
                        format!(
                            "In {sorted:?}, find the first zero-based index of {target}. Return JSON null if absent."
                        ),
                    )
                }
                _ => (
                    "max_subarray",
                    Task::MaxSubarray {
                        seed: task_seed,
                        items: *items,
                    },
                    format!(
                        "For {values:?}, return a maximum-sum nonempty contiguous interval as JSON with start, end_exclusive and sum."
                    ),
                ),
            };
            let generated = aether_runner::execute_worker(
                &json!({"version":WORKER_VERSION,"task":task}).to_string(),
            )?;
            let envelope: Value = serde_json::from_slice(&generated)?;
            let reference: Output = serde_json::from_value(envelope["output"].clone())?;
            evaluate(&task, &reference)?;
            let canonical = match &reference {
                Output::Sort { values } => json!(values),
                Output::Search { index } => json!(index),
                Output::MaxSubarray { candidate, .. } => json!(candidate),
                Output::Classification { .. } => {
                    return Err("unexpected reviewed task output".into());
                }
            };
            if let Some(submitted) = &submission {
                let raw = submitted.answer.clone();
                let proposed = match reference {
                    Output::Sort { .. } => {
                        serde_json::from_value(raw).map(|values| Output::Sort { values })
                    }
                    Output::Search { .. } => {
                        serde_json::from_value(raw).map(|index| Output::Search { index })
                    }
                    Output::MaxSubarray { baseline, .. } => {
                        serde_json::from_value(raw).map(|candidate| Output::MaxSubarray {
                            baseline,
                            candidate,
                        })
                    }
                    _ => return Err("unexpected reviewed task output".into()),
                };
                let Ok(proposed) = proposed else {
                    return Ok(Err(Rejection::MalformedAnswer));
                };
                if evaluate(&task, &proposed).is_err() {
                    return Ok(Err(Rejection::IncorrectAnswer));
                }
            }
            (
                "algorithmic_reasoning",
                family.into(),
                case_key("generated", usize::from(*case)),
                input,
                canonical,
                "independently_verified",
            )
        }
        _ => {
            let Some((topic, family, input, output)) = simulated(&recipe) else {
                return Ok(Err(Rejection::InvalidCase));
            };
            let case = format!("{family}/{}", sha256(input.as_bytes()));
            (topic, family.into(), case, input, output, "rule_checked")
        }
    };
    if verification != "independently_verified"
        && submission
            .as_ref()
            .is_some_and(|value| value.answer != ideal)
    {
        return Ok(Err(Rejection::IncorrectAnswer));
    }
    input = match variant {
        1 => format!("Please {input}"),
        2 => format!("  {input}  "),
        _ => input,
    };
    let origin = if submission
        .as_ref()
        .is_some_and(|s| s.human_authored_attested)
    {
        "human_authored_attested"
    } else if evidence.is_some() {
        "extracted"
    } else {
        "synthetic"
    };
    let output = if let Value::String(text) = ideal {
        text
    } else {
        serde_json::to_string(&ideal)?
    };
    Ok(Ok(Checked {
        input,
        output,
        topic,
        family,
        case,
        origin,
        verification,
        evidence,
    }))
}

type ResultRow = std::result::Result<Checked, Rejection>;
