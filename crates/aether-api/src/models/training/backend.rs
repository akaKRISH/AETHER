use aether_runner::sha256;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{BaseFormat, Manifest, REHEARSAL, Registry, SCHEMA};
use crate::{
    Result,
    lab::{Lab, valid_digest},
};

#[derive(Serialize)]
struct WorkerJob<'a> {
    schema: &'static str,
    run_kind: &'static str,
    manifest_sha256: &'a str,
    backend: &'static str,
    base_sha256: &'a str,
    train_sha256: &'a str,
    validation_sha256: &'a str,
    settings: &'a super::Settings,
    resource_caps: &'a super::Resources,
    planned_optimizer_steps: u64,
    cancel_before_or_at_step: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    step: u64,
    job_sha256: String,
    receipt_sha256: String,
    synthetic: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Completed,
    Cancelled,
}

trait TrainingBackend {
    fn run(
        &self,
        job: &WorkerJob<'_>,
        emit: &mut dyn FnMut(Checkpoint) -> Result<()>,
    ) -> Result<Outcome>;
}

struct SyntheticBackend;

fn schedule(steps: u64, every: u64) -> impl Iterator<Item = u64> {
    (1..=steps).filter(move |step| step.is_multiple_of(every) || *step == steps)
}

fn checkpoint_digest(job: &str, step: u64) -> String {
    sha256(format!("{REHEARSAL}/{job}/{step}").as_bytes())
}

impl TrainingBackend for SyntheticBackend {
    fn run(
        &self,
        job: &WorkerJob<'_>,
        emit: &mut dyn FnMut(Checkpoint) -> Result<()>,
    ) -> Result<Outcome> {
        let job_sha256 = job_digest(job)?;
        for step in schedule(
            job.planned_optimizer_steps,
            job.settings.checkpoint_every_steps,
        ) {
            if job
                .cancel_before_or_at_step
                .is_some_and(|limit| limit <= step)
            {
                return Ok(Outcome::Cancelled);
            }
            emit(Checkpoint {
                step,
                receipt_sha256: checkpoint_digest(&job_sha256, step),
                job_sha256: job_sha256.clone(),
                synthetic: true,
            })?;
        }
        Ok(Outcome::Completed)
    }
}

fn job_digest(job: &WorkerJob<'_>) -> Result<String> {
    Ok(sha256(&serde_json::to_vec(&serde_json::to_value(job)?)?))
}

fn run(backend: &impl TrainingBackend, job: &WorkerJob<'_>) -> Result<Value> {
    let job_sha256 = job_digest(job)?;
    let mut expected = schedule(
        job.planned_optimizer_steps,
        job.settings.checkpoint_every_steps,
    )
    .take_while(|step| {
        job.cancel_before_or_at_step
            .is_none_or(|limit| *step < limit)
    });
    let mut checkpoints = Vec::new();
    let outcome = backend.run(job, &mut |checkpoint| {
        if expected.next() != Some(checkpoint.step)
            || checkpoint.job_sha256 != job_sha256
            || !checkpoint.synthetic
            || checkpoint.receipt_sha256 != checkpoint_digest(&job_sha256, checkpoint.step)
        {
            return Err(
                "synthetic checkpoint failed manifest, sequence or integrity validation".into(),
            );
        }
        checkpoints.push(checkpoint);
        Ok(())
    })?;
    let terminal = if job.cancel_before_or_at_step.is_some() {
        Outcome::Cancelled
    } else {
        Outcome::Completed
    };
    if expected.next().is_some() || outcome != terminal {
        return Err("unexpected or incomplete synthetic worker result".into());
    }
    Ok(json!({
        "schema":"aether.synthetic-training-receipt/1", "manifest_sha256":job.manifest_sha256,
        "approval":"exact_digest_for_synthetic_rehearsal_only", "synthetic":true, "outcome":outcome,
        "job":job, "job_sha256":job_sha256, "checkpoints":checkpoints,
        "actual_weight_updates":0, "adapter_artifact_sha256":null, "regression_gates_status":"unmeasured"
    }))
}

pub(crate) fn rehearse(
    lab: &Lab,
    manifest_sha256: &str,
    approved_sha256: &str,
    cancel: Option<u64>,
) -> Result<Value> {
    if !valid_digest(manifest_sha256) || manifest_sha256 != approved_sha256 {
        return Err("explicit approval must match the exact stored manifest SHA-256".into());
    }
    let bytes = lab.artifact(manifest_sha256)?;
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    if sha256(&bytes) != manifest_sha256
        || manifest.schema != SCHEMA
        || !matches!(manifest.base.format, BaseFormat::SyntheticFixture)
        || manifest.python.is_some()
    {
        return Err("rehearsal accepts only frozen synthetic fixture manifests".into());
    }
    let steps = manifest.estimate.planned_optimizer_steps;
    if !(1..=16_384).contains(&steps)
        || !(1..=steps).contains(&manifest.settings.checkpoint_every_steps)
        || cancel.is_some_and(|limit| limit > steps)
    {
        return Err("invalid synthetic checkpoint schedule or cancellation point".into());
    }
    let job = WorkerJob {
        schema: "aether.training-worker-job/1",
        run_kind: "synthetic",
        manifest_sha256,
        backend: REHEARSAL,
        base_sha256: &manifest.base.identity.sha256,
        train_sha256: &manifest.dataset.train.sha256,
        validation_sha256: &manifest.dataset.validation.sha256,
        settings: &manifest.settings,
        resource_caps: &manifest.caps,
        planned_optimizer_steps: steps,
        cancel_before_or_at_step: cancel,
    };
    let mut receipt = run(&SyntheticBackend, &job)?;
    let hash = lab.record_development_artifact(&receipt)?;
    receipt["artifact_sha256"] = json!(hash);
    let registry = lab.model_registry()?;
    Registry::save(
        &registry.0,
        "training_receipt",
        manifest_sha256,
        &hash,
        &receipt,
    )?;
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct BadBackend(bool);

    impl TrainingBackend for BadBackend {
        fn run(
            &self,
            job: &WorkerJob<'_>,
            emit: &mut dyn FnMut(Checkpoint) -> Result<()>,
        ) -> Result<Outcome> {
            if self.0 {
                let digest = job_digest(job)?;
                emit(Checkpoint {
                    step: 1,
                    receipt_sha256: checkpoint_digest(&digest, 1),
                    job_sha256: digest,
                    synthetic: false,
                })?;
            }
            Ok(Outcome::Completed)
        }
    }

    #[test]
    fn rejects_incomplete_and_non_synthetic_worker_receipts() {
        let settings = serde_json::from_value(json!({
            "method":"lora","rank":1,"alpha":2,"dropout_bps":0,"targets":[],
            "optimizer":"adamw","learning_rate_ppb":100,"sequence_length":8,"micro_batch":1,
            "gradient_accumulation":1,"epochs":1,"seed":19,"checkpoint_every_steps":1,"keep_checkpoints":1
        })).unwrap();
        let caps = serde_json::from_value(
            json!({"ram_bytes":1,"vram_bytes":1,"storage_bytes":1,"dense_reference_flops":1}),
        )
        .unwrap();
        let job = WorkerJob {
            schema: "aether.training-worker-job/1",
            run_kind: "synthetic",
            manifest_sha256: "fixture",
            backend: REHEARSAL,
            base_sha256: "fixture",
            train_sha256: "fixture",
            validation_sha256: "fixture",
            settings: &settings,
            resource_caps: &caps,
            planned_optimizer_steps: 1,
            cancel_before_or_at_step: None,
        };
        assert!(run(&SyntheticBackend, &job).is_ok());
        assert!(
            run(&BadBackend(false), &job)
                .unwrap_err()
                .to_string()
                .contains("incomplete")
        );
        assert!(
            run(&BadBackend(true), &job)
                .unwrap_err()
                .to_string()
                .contains("integrity validation")
        );
    }
}
