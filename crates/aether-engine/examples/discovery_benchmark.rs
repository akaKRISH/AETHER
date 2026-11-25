use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::time::Instant;

use aether_engine::evolution::comparison::{CAPS, Case, Oracle, SearchArm, Trial, Verification};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

const SCOPE: &str = "aether.knapsack-matched-development/1";
const MAX_REPORT_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Serialize)]
struct Record {
    arm: SearchArm,
    evaluation_budget: u16,
    trial: Trial,
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

fn generate() -> Result<Export, Box<dyn Error>> {
    let mut phases_by_instance = Vec::new();
    let mut instances = Vec::new();
    for case in Case::development_grid() {
        let mut search_ns = Vec::new();
        let mut verify_ns = Vec::new();
        let mut trials = Vec::new();
        for cap in CAPS {
            for arm in SearchArm::ALL {
                let (trial, nanos) = timed(|| case.search(arm, cap));
                search_ns.push(nanos);
                trials.push((cap, arm, trial?));
            }
        }
        let mut checks = Vec::new();
        for (_, _, trial) in &trials {
            let (check, nanos) = timed(|| case.verify(trial));
            verify_ns.push(nanos);
            checks.push(check?);
        }
        // All paired searches are complete before the oracle runs.
        let (oracle, oracle_ns) = timed(|| case.oracle());
        let records = trials
            .into_iter()
            .zip(checks)
            .map(|((evaluation_budget, arm, trial), verification)| {
                let optimality_gap = oracle
                    .optimal_value
                    .checked_sub(trial.best_value())
                    .ok_or("candidate value exceeds oracle")?;
                Ok::<_, Box<dyn Error>>(Record {
                    arm,
                    evaluation_budget,
                    trial,
                    verification,
                    optimality_gap,
                })
            })
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
        scope: SCOPE.into(),
        evidence,
        evidence_sha256: format!("{:x}", Sha256::digest(&bytes)),
        phases_by_instance,
        evidence_bytes: bytes.len(),
        evidence_encode_ns,
    })
}

fn verify(saved: &Export, replay: &Export) -> Result<(), Box<dyn Error>> {
    if saved.scope != SCOPE || saved.evidence != replay.evidence {
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
        || saved.phases_by_instance.iter().any(|row| {
            row.search_ns.len() != CAPS.len() * SearchArm::ALL.len()
                || row.verify_ns.len() != row.search_ns.len()
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
        [command, path] if command == "run" => {
            let report = generate()?;
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
        [command, path] if command == "verify" => {
            let mut bytes = Vec::new();
            File::open(path)?
                .take(MAX_REPORT_BYTES + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX_REPORT_BYTES {
                return Err("export exceeds size limit".into());
            }
            let saved: Export = serde_json::from_slice(&bytes)?;
            let replay = generate()?;
            verify(&saved, &replay)?;
            println!(
                "Verified matched development evidence SHA-256 {}",
                saved.evidence_sha256
            );
        }
        _ => return Err("usage: discovery_benchmark <run|verify> <report.json>".into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_grid_replays_and_mismatched_exports_are_rejected() {
        let replay = generate().unwrap();
        let mut saved: Export =
            serde_json::from_slice(&serde_json::to_vec(&generate().unwrap()).unwrap()).unwrap();
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
