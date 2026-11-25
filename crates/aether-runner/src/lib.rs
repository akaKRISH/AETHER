#![forbid(unsafe_code)]

//! Process supervision for the built-in, reviewed native tasks.

mod optimization;
#[cfg(windows)]
mod windows_worker;

pub use optimization::{Interval, OptimizationComparison, optimization_input};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
#[cfg(windows)]
pub use windows_worker::wait_for_admission;

pub const WORKER_FLAG: &str = "__aether-builtin-worker-v1";
pub const WORKER_VERSION: &str = "aether.native-worker/1";
pub const MAX_OUTPUT: u64 = 4096;
const MAX_ITEMS: u16 = 128;
static ATTEMPT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortAlgorithm {
    Insertion,
    Selection,
    Standard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Task {
    Classification,
    Sort {
        seed: u32,
        items: u16,
        algorithm: SortAlgorithm,
    },
    Search {
        seed: u32,
        items: u16,
        target: i16,
    },
    MaxSubarray {
        seed: u32,
        items: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Output {
    Classification {
        predictions: Vec<u8>,
    },
    Sort {
        values: Vec<i16>,
    },
    Search {
        index: Option<usize>,
    },
    MaxSubarray {
        baseline: Interval,
        candidate: Interval,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Job {
    version: String,
    task: Task,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    version: String,
    task: Task,
    output: Output,
}

#[derive(Debug, Clone, Serialize)]
pub struct Measurement {
    pub output: Output,
    pub wall_ms: u64,
    pub output_bytes: u64,
    pub artifact_sha256: String,
    pub worker_pid: u32,
    #[serde(skip)]
    pub artifact: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("task exceeds the reviewed input or worker bounds")]
    Limits,
    #[error("reviewed task was cancelled")]
    Cancelled,
    #[error("reviewed task timed out")]
    Timeout,
    #[error("worker failed ({0})")]
    Worker(String),
    #[error("worker receipt failed identity or protocol validation")]
    Protocol,
    #[error("worker output disagrees with the trusted evaluator")]
    Evidence,
    #[error("worker I/O: {0}")]
    Io(#[from] io::Error),
    #[error("worker JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

impl Task {
    pub fn validate(&self) -> Result<(), Error> {
        match *self {
            Self::Classification => Ok(()),
            Self::Sort { items, .. }
            | Self::Search { items, .. }
            | Self::MaxSubarray { items, .. }
                if (1..=MAX_ITEMS).contains(&items) =>
            {
                Ok(())
            }
            _ => Err(Error::Limits),
        }
    }
}

fn data(seed: u32, items: u16) -> Vec<i16> {
    let mut state = seed;
    (0..items)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 24) as i16 - 128
        })
        .collect()
}

pub fn execute_worker(request: &str) -> Result<Vec<u8>, Error> {
    if request.len() > 512 {
        return Err(Error::Limits);
    }
    let job: Job = serde_json::from_str(request)?;
    if job.version != WORKER_VERSION {
        return Err(Error::Protocol);
    }
    job.task.validate()?;
    let output = match job.task {
        Task::Classification => {
            let (_, predictions) = aether_core::fixture();
            Output::Classification {
                predictions: predictions.to_vec(),
            }
        }
        Task::Sort {
            seed,
            items,
            algorithm,
        } => {
            let mut values = data(seed, items);
            match algorithm {
                SortAlgorithm::Insertion => {
                    for position in 1..values.len() {
                        let value = values[position];
                        let mut cursor = position;
                        while cursor > 0 && values[cursor - 1] > value {
                            values[cursor] = values[cursor - 1];
                            cursor -= 1;
                        }
                        values[cursor] = value;
                    }
                }
                SortAlgorithm::Selection => {
                    for position in 0..values.len() {
                        let mut minimum = position;
                        for cursor in position + 1..values.len() {
                            if values[cursor] < values[minimum] {
                                minimum = cursor;
                            }
                        }
                        values.swap(position, minimum);
                    }
                }
                SortAlgorithm::Standard => values.sort(),
            }
            Output::Sort { values }
        }
        Task::Search {
            seed,
            items,
            target,
        } => {
            let mut values = data(seed, items);
            values.sort();
            let position = values.partition_point(|value| *value < target);
            Output::Search {
                index: values
                    .get(position)
                    .filter(|v| **v == target)
                    .map(|_| position),
            }
        }
        Task::MaxSubarray { seed, items } => {
            let (baseline, candidate) = optimization::solve(&optimization_input(seed, items)?);
            Output::MaxSubarray {
                baseline,
                candidate,
            }
        }
    };
    Ok(serde_json::to_vec(&Receipt {
        version: job.version,
        task: job.task,
        output,
    })?)
}

/// Validate the output against an independently calculated reference.
/// The worker envelope carries no authoritative metrics or verdict.
pub fn evaluate(task: &Task, output: &Output) -> Result<(usize, usize), Error> {
    task.validate()?;
    match (task, output) {
        (Task::Classification, Output::Classification { predictions }) => {
            let (labels, _) = aether_core::fixture();
            let accuracy =
                aether_core::evaluate(labels, predictions).map_err(|_| Error::Evidence)?;
            Ok((accuracy.correct, accuracy.total))
        }
        (Task::Sort { seed, items, .. }, Output::Sort { values }) => {
            let mut reference = data(*seed, *items);
            reference.sort_unstable();
            if *values != reference {
                return Err(Error::Evidence);
            }
            Ok((reference.len(), reference.len()))
        }
        (
            Task::Search {
                seed,
                items,
                target,
            },
            Output::Search { index },
        ) => {
            let mut reference = data(*seed, *items);
            reference.sort_unstable();
            if *index != reference.iter().position(|value| value == target) {
                return Err(Error::Evidence);
            }
            Ok((1, 1))
        }
        (Task::MaxSubarray { .. }, Output::MaxSubarray { .. }) => {
            let comparison = evaluate_optimization(task, output)?;
            if comparison.candidate_sum != comparison.reference_sum {
                return Err(Error::Evidence);
            }
            Ok((1, 1))
        }
        _ => Err(Error::Protocol),
    }
}

/// Independently evaluate the baseline and candidate from the reviewed optimization receipt.
pub fn evaluate_optimization(
    task: &Task,
    output: &Output,
) -> Result<OptimizationComparison, Error> {
    match (task, output) {
        (
            Task::MaxSubarray { seed, items },
            Output::MaxSubarray {
                baseline,
                candidate,
            },
        ) => optimization::compare(&optimization_input(*seed, *items)?, baseline, candidate),
        _ => Err(Error::Protocol),
    }
}

/// Run the fixed worker subcommand of the trusted platform executable.
/// The reviewed worker implementation starts no descendants.
pub fn supervise(
    platform_exe: &Path,
    task: &Task,
    timeout: Duration,
    cancelled: &AtomicBool,
) -> Result<Measurement, Error> {
    task.validate()?;
    if cancelled.load(Ordering::Acquire) {
        return Err(Error::Cancelled);
    }
    if timeout.is_zero() || timeout > Duration::from_secs(30) {
        return Err(Error::Limits);
    }

    let request = serde_json::to_string(&Job {
        version: WORKER_VERSION.into(),
        task: task.clone(),
    })?;
    let directory = std::env::temp_dir().join(format!(
        "aether-worker-{}-{}",
        std::process::id(),
        ATTEMPT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory)?;
    let result = supervise_child(platform_exe, task, timeout, cancelled, request, &directory);
    let cleanup = fs::remove_dir(&directory);
    let result = result?;
    cleanup?;
    Ok(result)
}

fn supervise_child(
    platform_exe: &Path,
    task: &Task,
    timeout: Duration,
    cancelled: &AtomicBool,
    request: String,
    directory: &Path,
) -> Result<Measurement, Error> {
    let mut command = Command::new(platform_exe);
    command
        .arg(WORKER_FLAG)
        .arg(request)
        .env_clear()
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let started = Instant::now();
    #[cfg(windows)]
    let (mut child, containment) = windows_worker::spawn(&mut command)?;
    #[cfg(not(windows))]
    let mut child = command.spawn()?;
    let pid = child.id();
    let output = child.stdout.take().ok_or(Error::Protocol)?;
    let reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        output.take(MAX_OUTPUT + 1).read_to_end(&mut bytes)?;
        Ok::<_, io::Error>(bytes)
    });
    let outcome = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status.success() {
                    Ok(())
                } else {
                    Err(Error::Worker(status.to_string()))
                };
            }
            Ok(None) => {}
            Err(error) => break Err(error.into()),
        }
        if cancelled.load(Ordering::Acquire) {
            break Err(Error::Cancelled);
        }
        if started.elapsed() >= timeout {
            break Err(Error::Timeout);
        }
        thread::sleep(Duration::from_millis(2));
    };
    #[cfg(windows)]
    drop(containment);
    if outcome.is_err() {
        // A process can exit between try_wait and kill.
        let _ = child.kill();
        child.wait()?;
    }
    let artifact = reader.join().map_err(|_| Error::Protocol)??;
    outcome?;
    if artifact.len() as u64 > MAX_OUTPUT {
        return Err(Error::Limits);
    }
    let receipt: Receipt = serde_json::from_slice(&artifact)?;
    if receipt.version != WORKER_VERSION || &receipt.task != task {
        return Err(Error::Protocol);
    }
    let wall_ms =
        u64::try_from(started.elapsed().as_nanos().div_ceil(1_000_000)).unwrap_or(u64::MAX);
    Ok(Measurement {
        output: receipt.output,
        wall_ms,
        output_bytes: artifact.len() as u64,
        artifact_sha256: sha256(&artifact),
        worker_pid: pid,
        artifact,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_algorithms_match_the_reference_across_seeds() {
        for seed in 0..32 {
            for algorithm in [
                SortAlgorithm::Insertion,
                SortAlgorithm::Selection,
                SortAlgorithm::Standard,
            ] {
                let task = Task::Sort {
                    seed,
                    items: 64,
                    algorithm,
                };
                let request = serde_json::to_string(&Job {
                    version: WORKER_VERSION.into(),
                    task: task.clone(),
                })
                .unwrap();
                let actual = execute_worker(&request).unwrap();
                assert_eq!(actual, execute_worker(&request).unwrap());
                let receipt: Receipt = serde_json::from_slice(&actual).unwrap();
                assert_eq!(evaluate(&task, &receipt.output).unwrap(), (64, 64));
            }
        }
    }

    #[test]
    fn malformed_and_false_evidence_are_rejected() {
        assert!(execute_worker(r#"{"version":"old","task":{"kind":"classification"}}"#).is_err());
        let task = Task::Sort {
            seed: 1,
            items: 4,
            algorithm: SortAlgorithm::Standard,
        };
        assert!(matches!(
            evaluate(&task, &Output::Sort { values: vec![0; 4] }),
            Err(Error::Evidence)
        ));
    }

    #[test]
    fn expired_admission_never_launches_the_worker() {
        let path = Path::new("missing-aether-executable");
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            supervise(
                path,
                &Task::Classification,
                Duration::from_secs(1),
                &cancelled
            ),
            Err(Error::Cancelled)
        ));
    }
}
