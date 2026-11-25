use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::time::Instant;

use aether_engine::evolution::comparison::{
    CAPS, Case, HybridArm, HybridTrace, Oracle, SearchArm, Trial, Verification,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

const SCOPE: &str = "aether.knapsack-matched-development/1";
const MAX_REPORT_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone, Copy, Serialize)]
#[serde(untagged)]
enum Arm {
    Baseline(SearchArm),
    Hybrid(HybridArm),
}

#[derive(Serialize)]
struct Record {
    arm: Arm,
    evaluation_budget: u16,
    trial: Trial,
    #[serde(skip_serializing_if = "Option::is_none")]
    operations: Option<HybridTrace>,
    verification: Verification,
    optimality_gap: u16,
}

#[derive(Serialize)]
struct Instance {
    case: Case,
    oracle: Oracle,
    records: Vec<Record>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseTimes {
    search_ns: Vec<u64>,
    verify_ns: Vec<u64>,
    oracle_ns: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Export {
    scope: String,
    evidence: Value,
    evidence_sha256: String,
    phases_by_instance: Vec<PhaseTimes>,
    evidence_bytes: usize,
    evidence_encode_ns: u64,
}

fn timed<T>(work: impl FnOnce() -> T) -> (T, u64) {
    let start = Instant::now();
    let result = work();
    (result, start.elapsed().as_nanos() as u64)
}

fn generate(hybrid: bool) -> Result<Export, Box<dyn Error>> {
    let mut phases_by_instance = Vec::new();
    let mut instances = Vec::new();
    let mut arms: Vec<_> = SearchArm::ALL.into_iter().map(Arm::Baseline).collect();
    let cases: Vec<_> = if hybrid {
        arms.extend(HybridArm::ALL.into_iter().map(Arm::Hybrid));
        Case::hybrid_grid().collect()
    } else {
        Case::development_grid().collect()
    };
    for (instance_index, case) in cases.into_iter().enumerate() {
        let mut search_ns = Vec::new();
        let mut verify_ns = Vec::new();
        let mut trials = Vec::new();
        for (cap_index, cap) in CAPS.into_iter().enumerate() {
            let offset = if hybrid {
                (instance_index + cap_index) % arms.len()
            } else {
                0
            };
            let mut results = Vec::with_capacity(arms.len());
            for k in 0..arms.len() {
                let index = (k + offset) % arms.len();
                let arm = arms[index];
                let (result, nanos) = timed(|| match arm {
                    Arm::Baseline(arm) => case.search(arm, cap).map(|trial| (trial, None)),
                    Arm::Hybrid(arm) => case
                        .search_hybrid(arm, cap)
                        .map(|(trial, operations)| (trial, Some(operations))),
                });
                results.push((index, (cap, arm, result?, nanos)));
            }
            results.sort_unstable_by_key(|row| row.0);
            for (_, (cap, arm, (trial, operations), nanos)) in results {
                search_ns.push(nanos);
                trials.push((cap, arm, trial, operations));
            }
        }
        let mut checks = Vec::new();
        for (_, _, trial, _) in &trials {
            let (check, nanos) = timed(|| case.verify(trial));
            verify_ns.push(nanos);
            checks.push(check?);
        }
        // All paired searches are complete before the oracle runs.
        let (oracle, oracle_ns) = timed(|| case.oracle());
        let records = trials
            .into_iter()
            .zip(checks)
            .map(
                |((evaluation_budget, arm, trial, operations), verification)| {
                    let optimality_gap = oracle
                        .optimal_value
                        .checked_sub(trial.best_value())
                        .ok_or("candidate value exceeds oracle")?;
                    Ok::<_, Box<dyn Error>>(Record {
                        arm,
                        evaluation_budget,
                        trial,
                        operations,
                        verification,
                        optimality_gap,
                    })
                },
            )
            .collect::<Result<_, _>>()?;
        instances.push(Instance {
            case,
            oracle,
            records,
        });
        phases_by_instance.push(PhaseTimes {
            search_ns,
            verify_ns,
            oracle_ns,
        });
    }
    let (encoded, evidence_encode_ns) = timed(|| {
        let evidence = serde_json::to_value(&instances)?;
        let bytes = serde_json::to_vec(&evidence)?;
        Ok::<_, serde_json::Error>((evidence, bytes))
    });
    let (evidence, bytes) = encoded?;
    Ok(Export {
        scope: if hybrid {
            "aether.knapsack-hybrid-development/1"
        } else {
            SCOPE
        }
        .into(),
        evidence,
        evidence_sha256: format!("{:x}", Sha256::digest(&bytes)),
        phases_by_instance,
        evidence_bytes: bytes.len(),
        evidence_encode_ns,
    })
}

fn verify(saved: &Export, replay: &Export) -> Result<(), Box<dyn Error>> {
    if saved.scope != replay.scope || saved.evidence != replay.evidence {
        return Err(
            "export differs from the expected paired grid or deterministic outcomes".into(),
        );
    }
    if saved.evidence_sha256 != replay.evidence_sha256
        || saved.evidence_bytes != replay.evidence_bytes
    {
        return Err("export evidence digest or byte count differs".into());
    }
    if saved.phases_by_instance.len() != replay.phases_by_instance.len()
        || saved
            .phases_by_instance
            .iter()
            .zip(&replay.phases_by_instance)
            .any(|(row, expected)| {
                row.search_ns.len() != expected.search_ns.len()
                    || row.verify_ns.len() != expected.verify_ns.len()
            })
    {
        return Err("export timing groups do not match the paired grid".into());
    }
    for instance in replay
        .evidence
        .as_array()
        .ok_or("missing replay instances")?
    {
        let records = instance["records"]
            .as_array()
            .ok_or("missing replay records")?;
        if records
            .iter()
            .any(|record| record["verification"]["passed"] != true)
        {
            return Err("independent incumbent check failed".into());
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command, path] if matches!(command.as_str(), "run" | "hybrid-run") => {
            let report = generate(command == "hybrid-run")?;
            let (written, write_ns) = timed(|| {
                let file = File::options().write(true).create_new(true).open(path)?;
                let mut output = io::BufWriter::new(file);
                serde_json::to_writer(&mut output, &report)?;
                output.write_all(b"\n")?;
                output.flush()?;
                Ok::<_, Box<dyn Error>>(output.get_ref().metadata()?.len())
            });
            println!(
                "Wrote {} bytes in {write_ns} ns (serialization and buffered flush)",
                written?
            );
        }
        [command, path] if matches!(command.as_str(), "verify" | "hybrid-verify") => {
            let mut bytes = Vec::new();
            File::open(path)?
                .take(MAX_REPORT_BYTES + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX_REPORT_BYTES {
                return Err("export exceeds size limit".into());
            }
            let saved: Export = serde_json::from_slice(&bytes)?;
            let replay = generate(command == "hybrid-verify")?;
            verify(&saved, &replay)?;
            println!(
                "Verified matched development evidence SHA-256 {}",
                saved.evidence_sha256
            );
        }
        _ => {
            return Err(
                "usage: discovery_benchmark <run|verify|hybrid-run|hybrid-verify> <report.json>"
                    .into(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_grid_replays_without_changing_prior_baseline_rows() {
        let old = generate(false).unwrap();
        let new = generate(true).unwrap();
        let mut saved = generate(true).unwrap();
        verify(&saved, &new).unwrap();
        assert!(verify(&saved, &old).is_err());
        for (original, extended) in old
            .evidence
            .as_array()
            .unwrap()
            .iter()
            .zip(new.evidence.as_array().unwrap())
        {
            assert_eq!(original["case"], extended["case"]);
            assert_eq!(original["oracle"], extended["oracle"]);
            let records = extended["records"].as_array().unwrap();
            let baselines: Vec<_> = records
                .iter()
                .filter(|row| row.get("operations").is_none())
                .cloned()
                .collect();
            assert_eq!(original["records"], Value::Array(baselines));
        }
        saved.evidence[0]["records"][3]["operations"]["operations"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(verify(&saved, &new).is_err());
    }

    #[test]
    fn complete_grid_replays_and_mismatched_exports_are_rejected() {
        let replay = generate(false).unwrap();
        let mut saved: Export =
            serde_json::from_slice(&serde_json::to_vec(&generate(false).unwrap()).unwrap())
                .unwrap();
        verify(&saved, &replay).unwrap();
        let records = saved.evidence[0]["records"].as_array_mut().unwrap();
        let record = records.pop().unwrap();
        assert!(verify(&saved, &replay).is_err());
        let records = saved.evidence[0]["records"].as_array_mut().unwrap();
        records.push(records[0].clone());
        assert!(verify(&saved, &replay).is_err());
        let records = saved.evidence[0]["records"].as_array_mut().unwrap();
        *records.last_mut().unwrap() = record;
        saved.evidence[0]["records"][5]["evaluation_budget"] = 8.into();
        assert!(verify(&saved, &replay).is_err());
        saved.evidence[0]["records"][5]["evaluation_budget"] = 24.into();
        saved.evidence_sha256 = "bad".into();
        assert!(verify(&saved, &replay).is_err());
        saved.evidence_sha256.clone_from(&replay.evidence_sha256);
        saved.phases_by_instance[0].verify_ns.pop();
        assert!(verify(&saved, &replay).is_err());
    }
}
