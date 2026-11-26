mod backend;
mod dataset;

use std::{collections::BTreeSet, path::Path, sync::atomic::AtomicBool};

use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{MIB, Registry};
use crate::{
    Result,
    lab::{Lab, read_json_file, valid_digest},
};
use dataset::{Dataset, DatasetInput, valid_text};

const SCHEMA: &str = "aether.training-manifest/1";
const REHEARSAL: &str = "aether.synthetic-training-worker/1";

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Method {
    Lora,
    Qlora,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum BaseFormat {
    Gguf,
    SafetensorsShard,
    SyntheticFixture,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Geometry {
    hidden: u64,
    layers: u64,
    attention_heads: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BaseInput {
    sha256: String,
    architecture: String,
    license: String,
    provenance: String,
    geometry: Geometry,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Base {
    identity: BaseInput,
    format: BaseFormat,
    weight_bytes: u64,
    stored_elements: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Target {
    name: String,
    in_features: u64,
    out_features: u64,
    instances: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Optimizer {
    Adamw,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    method: Method,
    rank: u64,
    alpha: u64,
    dropout_bps: u16,
    targets: Vec<Target>,
    optimizer: Optimizer,
    learning_rate_ppb: u64,
    sequence_length: u64,
    micro_batch: u64,
    gradient_accumulation: u64,
    epochs: u64,
    seed: u32,
    checkpoint_every_steps: u64,
    keep_checkpoints: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Resources {
    ram_bytes: u64,
    vram_bytes: u64,
    storage_bytes: u64,
    dense_reference_flops: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PythonPins {
    worker_sha256: String,
    dependency_lock_sha256: String,
    interpreter_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposalRequest {
    base: BaseInput,
    dataset: DatasetInput,
    settings: Settings,
    caps: Resources,
    baseline_sha256: Option<String>,
    python: Option<PythonPins>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Baseline {
    report_sha256: String,
    evaluation_suite_sha256: String,
    evaluator_sha256: String,
    synthetic: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Estimate {
    resources: Resources,
    adapter_parameters: u64,
    planned_optimizer_steps: u64,
    padded_training_tokens: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    base: Base,
    dataset: Dataset,
    settings: Settings,
    quantization: String,
    caps: Resources,
    estimate: Estimate,
    baseline: Option<Baseline>,
    python: Option<PythonPins>,
    requirements: Vec<String>,
    regression_gates: Vec<String>,
}

fn product(values: &[u64]) -> Result<u64> {
    values.iter().try_fold(1u64, |a, b| {
        a.checked_mul(*b)
            .ok_or_else(|| "training estimate overflow".into())
    })
}

fn sum(values: &[u64]) -> Result<u64> {
    values.iter().try_fold(0u64, |a, b| {
        a.checked_add(*b)
            .ok_or_else(|| "training estimate overflow".into())
    })
}

fn estimate(base: &Base, data: &Dataset, settings: &Settings) -> Result<Estimate> {
    let Settings {
        rank,
        alpha,
        dropout_bps,
        targets,
        sequence_length: seq,
        micro_batch: batch,
        gradient_accumulation: accumulation,
        epochs,
        checkpoint_every_steps,
        keep_checkpoints,
        ..
    } = settings;
    let Geometry {
        hidden,
        layers,
        attention_heads,
    } = base.identity.geometry;
    if !(1..=256).contains(rank)
        || !(1..=1024).contains(alpha)
        || *dropout_bps > 5_000
        || !(1..=32).contains(&targets.len())
        || !(8..=8192).contains(seq)
        || !(1..=64).contains(batch)
        || !(1..=256).contains(accumulation)
        || !(1..=30).contains(epochs)
        || !(1..=8).contains(keep_checkpoints)
        || !valid_text(&base.identity.architecture, 100)
        || !valid_text(&base.identity.license, 120)
        || !valid_text(&base.identity.provenance, 300)
        || !(1..=65536).contains(&hidden)
        || !(1..=256).contains(&layers)
        || !(1..=256).contains(&attention_heads)
        || !hidden.is_multiple_of(attention_heads)
        || !(1..=10_000_000).contains(&settings.learning_rate_ppb)
        || base.stored_elements == 0
    {
        return Err("invalid or unsupported training settings".into());
    }
    let mut names = BTreeSet::new();
    let mut adapter_parameters = 0;
    for t in targets {
        if !valid_text(&t.name, 120)
            || !names.insert(&t.name)
            || !(*rank..=65536).contains(&t.in_features)
            || !(*rank..=65536).contains(&t.out_features)
            || !(1..=layers).contains(&t.instances)
        {
            return Err("invalid or duplicate adapter target geometry".into());
        }
        adapter_parameters = sum(&[
            adapter_parameters,
            product(&[*rank, sum(&[t.in_features, t.out_features])?, t.instances])?,
        ])?;
    }
    if adapter_parameters > base.stored_elements {
        return Err("adapter estimate exceeds the base element count".into());
    }
    let effective_batch = product(&[*batch, *accumulation])?;
    let steps = product(&[data.train.rows.div_ceil(effective_batch), *epochs])?;
    if !(1..=steps).contains(checkpoint_every_steps) || steps > 16_384 {
        return Err("invalid checkpoint cadence or training step limit".into());
    }
    let padded_training_tokens = product(&[steps, effective_batch, *seq])?;
    let adapter_state = product(&[adapter_parameters, 16])?;
    let base_bytes = match settings.method {
        Method::Lora => product(&[base.stored_elements, 2])?,
        Method::Qlora => sum(&[
            base.stored_elements.div_ceil(2),
            base.stored_elements.div_ceil(8),
        ])?,
    };
    let activation = product(&[*batch, *seq, hidden, layers, 16])?;
    let attention = product(&[*batch, *seq, *seq, layers, attention_heads, 4])?;
    Ok(Estimate {
        resources: Resources {
            ram_bytes: sum(&[base.weight_bytes.max(base_bytes), adapter_state, 2048 * MIB])?,
            vram_bytes: sum(&[base_bytes, adapter_state, activation, attention, 1024 * MIB])?,
            storage_bytes: sum(&[
                base.weight_bytes,
                data.train.bytes,
                data.validation.bytes,
                data.independent_test.bytes,
                product(&[adapter_state, *keep_checkpoints])?,
                product(&[adapter_parameters, 2])?,
                64 * MIB,
            ])?,
            dense_reference_flops: product(&[6, base.stored_elements, padded_training_tokens])?,
        },
        adapter_parameters,
        planned_optimizer_steps: steps,
        padded_training_tokens,
    })
}

fn resolve_base(registry: &Registry, identity: BaseInput) -> Result<Base> {
    if !valid_digest(&identity.sha256) {
        return Err("base model requires a lowercase SHA-256 digest".into());
    }
    let record: String = registry
        .0
        .query_row(
            "SELECT record FROM model_registry WHERE kind='file' AND key=?1 AND sha256=?1",
            [&identity.sha256],
            |r| r.get(0),
        )
        .optional()?
        .ok_or("base weights require an explicit models-inspect registry record")?;
    let record: Value = serde_json::from_str(&record)?;
    let inspection = &record["inspection"];
    let meta = &inspection["metadata"];
    let format = match inspection["format"].as_str() {
        Some("gguf") => BaseFormat::Gguf,
        Some("safetensors") => BaseFormat::SafetensorsShard,
        _ => return Err("unrecognized model inspection".into()),
    };
    for (seen, declared) in [
        (
            meta["general.architecture"]
                .as_str()
                .or(meta["architecture"].as_str()),
            &identity.architecture,
        ),
        (
            meta["general.license"]
                .as_str()
                .or(meta["license"].as_str()),
            &identity.license,
        ),
    ] {
        if seen.is_some_and(|value| !value.eq_ignore_ascii_case(declared)) {
            return Err(
                "base architecture or license disagrees with its inspected metadata".into(),
            );
        }
    }
    Ok(Base {
        identity,
        format,
        weight_bytes: inspection["file_bytes"]
            .as_u64()
            .ok_or("missing file size")?,
        stored_elements: inspection["stored_elements"]
            .as_u64()
            .ok_or("missing stored element count")?,
    })
}

fn baseline(lab: &Lab, base: &Base, reference: Option<String>) -> Result<Option<Baseline>> {
    reference
        .map(|report_sha256| {
            // Applying the existing matched-grid checks keeps the baseline contract intact.
            let comparison = super::evaluation::compare(lab, &report_sha256, &report_sha256)?;
            let report: Value = serde_json::from_slice(&lab.artifact(&report_sha256)?)?;
            let source = &report["manifest"];
            let synthetic = matches!(
                source["provider"].as_str(),
                Some("fixture" | "fixture-regression")
            );
            if synthetic != matches!(base.format, BaseFormat::SyntheticFixture)
                || (!synthetic && source["weight_sha256"] != base.identity.sha256)
                || !comparison["paired"].as_array().is_some_and(|rows| {
                    rows.iter().all(|r| r["baseline_passed"].is_boolean())
                        && rows
                            .iter()
                            .any(|r| r["case"] == "safety" && r["baseline_passed"] == true)
                })
            {
                return Err(
                    "baseline identity or safety gate does not match the proposed base".into(),
                );
            }
            Ok(Baseline {
                report_sha256,
                evaluation_suite_sha256: source["suite_sha256"]
                    .as_str()
                    .filter(|v| valid_digest(v))
                    .ok_or("missing suite pin")?
                    .into(),
                evaluator_sha256: source["evaluator_sha256"]
                    .as_str()
                    .filter(|v| valid_digest(v))
                    .ok_or("missing evaluator pin")?
                    .into(),
                synthetic,
            })
        })
        .transpose()
}

fn build(
    base: Base,
    dataset: Dataset,
    settings: Settings,
    caps: Resources,
    baseline: Option<Baseline>,
    python: Option<PythonPins>,
) -> Result<Manifest> {
    if [
        caps.ram_bytes,
        caps.vram_bytes,
        caps.storage_bytes,
        caps.dense_reference_flops,
    ]
    .contains(&0)
        || python.as_ref().is_some_and(|p| {
            [
                &p.worker_sha256,
                &p.dependency_lock_sha256,
                &p.interpreter_sha256,
            ]
            .iter()
            .any(|v| !valid_digest(v))
        })
    {
        return Err("invalid hard resource cap or Python backend pin".into());
    }
    let estimate = estimate(&base, &dataset, &settings)?;
    let quantization = match settings.method {
        Method::Lora => "fp16_frozen_base",
        Method::Qlora => "nf4_double_quantization_fp16_compute",
    };
    Ok(Manifest {
        schema: SCHEMA.into(), base, dataset, settings, quantization: quantization.into(), caps, estimate, baseline, python,
        requirements: [
            "Pin complete native checkpoint, tokenizer, config and dataset bytes. Mount base weights read-only and rehash before and after.",
            "Keep independent test prompts, labels and paths outside the worker. Only the independent reviewer releases evaluation.",
            "Pin reviewed worker, interpreter and dependencies. Disable network, remote code and external transfers.",
            "Use fixed shuffle and initialization seed; record runtime versions, device, precision, deterministic settings and data order.",
            "Publish adapter-only checkpoints atomically. Bind resume to manifest, data order, optimizer and RNG state.",
            "Cooperate with cancellation at batch boundaries and discard partial checkpoints on failure. Require bounded supervised shutdown.",
        ].into_iter().map(String::from).collect(),
        regression_gates: [
            "Match base and adapter on the pinned suite, evaluator, independent cases, seeds and resource caps.",
            "Preserve each baseline capability pass. Require safety and expected-abstention cases to pass.",
            "Require paired nonmissing calibration and no increase in Brier loss. Reject incomplete grids and unverified receipts.",
        ].into_iter().map(String::from).collect(),
    })
}

fn publish(lab: &Lab, manifest: &Manifest) -> Result<Value> {
    let sha = lab.record_development_artifact(manifest)?;
    let mut blockers = vec![
        "Real training execution is disabled for APO-06A.",
        "No approved, qualified Python training worker or runtime is registered.",
        "Read-only base mounting, checkpoint integrity and independent test custody require qualification.",
    ];
    blockers.push(match manifest.base.format {
        BaseFormat::Gguf => "GGUF inference weights are ineligible for this training backend.",
        BaseFormat::SafetensorsShard => {
            "One SafeTensors shard does not establish a complete trainable checkpoint."
        }
        BaseFormat::SyntheticFixture => "The fixture base and its test custody are synthetic.",
    });
    if manifest.baseline.is_none() {
        blockers.push("A complete matching, independently checked baseline report is missing.");
    }
    let ram = aether_runner::available_memory_bytes();
    let gpu = crate::hardware::gpu_snapshot();
    let vram = match gpu.devices.as_slice() {
        [device] => Some(u64::from(device.memory_free_mib) * MIB),
        _ => None,
    };
    let (estimate, caps) = (&manifest.estimate.resources, &manifest.caps);
    if estimate.ram_bytes > caps.ram_bytes
        || estimate.vram_bytes > caps.vram_bytes
        || estimate.storage_bytes > caps.storage_bytes
        || estimate.dense_reference_flops > caps.dense_reference_flops
    {
        blockers.push("Resource estimates exceed the declared caps.");
    }
    if [(ram, estimate.ram_bytes), (vram, estimate.vram_bytes)]
        .iter()
        .any(|(available, needed)| available.is_none_or(|value| value < *needed))
    {
        blockers.push("Host RAM or VRAM headroom is insufficient or unavailable.");
    }
    blockers
        .push("Disk reservation and measured training throughput require separate qualification.");
    let registry = lab.model_registry()?;
    let record = json!({"manifest_sha256":sha,"manifest":manifest,"approval":"unapproved", "real_training":"blocked",
        "feasibility":{"ram_available_bytes":ram,"vram_free_bytes":vram,"storage_available_bytes":null,
            "measurement_scope":"host snapshot, no training allocation or throughput measured","blockers":blockers}});
    Registry::save(&registry.0, "training_proposal", &sha, &sha, &record)?;
    Ok(record)
}

pub(crate) fn propose(lab: &Lab, path: &Path) -> Result<Value> {
    let request: ProposalRequest = read_json_file(path)?;
    let dataset = Dataset::read(path.parent().unwrap_or(Path::new(".")), request.dataset)?;
    let base = resolve_base(&lab.model_registry()?, request.base)?;
    let baseline = baseline(lab, &base, request.baseline_sha256)?;
    publish(
        lab,
        &build(
            base,
            dataset,
            request.settings,
            request.caps,
            baseline,
            request.python,
        )?,
    )
}

pub(crate) fn fixture(lab: &Lab, method: &str) -> Result<Value> {
    let method = match method {
        "lora" => Method::Lora,
        "qlora" => Method::Qlora,
        _ => return Err("expected lora or qlora".into()),
    };
    let base = Base {
        identity: BaseInput {
            sha256: aether_runner::sha256(b"AETHER synthetic base fixture v1"),
            architecture: "aether-fixture".into(),
            license: "CC0-1.0".into(),
            provenance: "AETHER synthetic arithmetic fixture, no weight file".into(),
            geometry: Geometry {
                hidden: 64,
                layers: 2,
                attention_heads: 4,
            },
        },
        format: BaseFormat::SyntheticFixture,
        weight_bytes: 65_536,
        stored_elements: 32_768,
    };
    let receipt = super::evaluation::evaluate(lab, "fixture", &AtomicBool::new(false))?;
    let baseline = baseline(
        lab,
        &base,
        receipt["artifact_sha256"].as_str().map(String::from),
    )?;
    let settings = Settings {
        method,
        rank: 4,
        alpha: 8,
        dropout_bps: 500,
        targets: vec![Target {
            name: "q_proj".into(),
            in_features: 64,
            out_features: 64,
            instances: 2,
        }],
        optimizer: Optimizer::Adamw,
        learning_rate_ppb: 100_000,
        sequence_length: 64,
        micro_batch: 1,
        gradient_accumulation: 1,
        epochs: 1,
        seed: 19,
        checkpoint_every_steps: 2,
        keep_checkpoints: 2,
    };
    let caps = Resources {
        ram_bytes: 4096 * MIB,
        vram_bytes: 4096 * MIB,
        storage_bytes: 1024 * MIB,
        dense_reference_flops: 1_000_000_000,
    };
    publish(
        lab,
        &build(base, Dataset::fixture()?, settings, caps, baseline, None)?,
    )
}

pub(crate) fn list(lab: &Lab) -> Result<Value> {
    let registry = lab.model_registry()?;
    let mut statement = registry
        .0
        .prepare("SELECT record FROM model_registry WHERE kind=?1 ORDER BY rowid DESC LIMIT 33")?;
    let mut query = |kind: &str| -> Result<Vec<Value>> {
        statement
            .query_map([kind], |row| row.get::<_, String>(0))?
            .map(|row| Ok(serde_json::from_str(&row?)?))
            .collect()
    };
    let mut proposals = query("training_proposal")?;
    let mut receipts = query("training_receipt")?;
    let truncated = proposals.len() > 32 || receipts.len() > 32;
    proposals.truncate(32);
    receipts.truncate(32);
    Ok(
        json!({"schema":"aether.training-registry/1","proposals":proposals,"receipts":receipts,"truncated":truncated}),
    )
}

pub(crate) use backend::rehearse;
pub(super) use dataset::validate_development_exports;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_estimate_arithmetic_rejects_overflow() {
        assert!(sum(&[u64::MAX, 1]).is_err());
        assert!(product(&[u64::MAX / 6 + 1, 6]).is_err());
    }
}
