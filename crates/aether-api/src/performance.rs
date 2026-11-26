//! Bounded, development-only native microbenchmarks; model inference is excluded.
use std::{
    fs,
    hint::black_box,
    net::TcpListener,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::atomic::AtomicBool,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use aether_engine::{
    Action, ActionKind, Budget, ResourceVec, ScheduleContext, ScoreWeights, discover,
    schedule_with_trace,
};
use aether_runner::{Task, supervise};
use aether_store::{BudgetUnits, NewCampaign, NewClaim, Store};
use serde::Serialize;
use serde_json::json;

use crate::{
    Result, inference,
    lab::{Lab, PlanRequest},
};

const WARMUP: usize = 5;
const SAMPLES: usize = 100;
const WORKERS: u16 = 4;

#[derive(Serialize)]
struct Metric {
    name: &'static str,
    warmup_iterations: usize,
    measured_iterations: usize,
    samples_us: Vec<f64>,
    p50_us: f64,
    p95_us: f64,
    p99_us: Option<f64>,
    median_ci95_us: [f64; 2],
    throughput_ops_per_second: f64,
}

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Fixtures {
    scheduler: Vec<Vec<Action>>,
    graphs: [Store; 2],
    sqlite: Store,
    lab: Lab,
    campaign: String,
    executable: PathBuf,
    api_url: String,
    token: String,
    _server: Server,
    _scratch: Scratch,
}

pub(super) fn run() -> Result<serde_json::Value> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let scratch =
        Scratch(std::env::temp_dir().join(format!("aether-perf-{}-{nonce}", std::process::id())));
    fs::create_dir_all(&scratch.0)?;
    let executable = std::env::current_exe()?;
    let graphs = [graph_fixture(64)?, graph_fixture(256)?];
    let sqlite = sqlite_fixture(scratch.0.join("sqlite-performance.db"))?;
    let lab = Lab::open(scratch.0.join("lab"))?;
    let campaign = "perf-report".to_owned();
    let created = lab.create(&PlanRequest::example(campaign.clone(), 19))?;
    lab.approve(&campaign, &created.spec.protocol_hash)?;
    lab.run(&campaign, &AtomicBool::new(false))?;

    let token = "aether-performance-local-token-000".to_owned();
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
    let port = listener.local_addr()?.port();
    drop(listener);
    let child = Command::new(&executable)
        .args(["serve", &port.to_string()])
        .env("AETHER_DATA_DIR", scratch.0.join("lab"))
        .env("AETHER_TOKEN", &token)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let server = Server(child);
    let api_url = format!("http://127.0.0.1:{port}");
    wait_for_api(&api_url)?;
    let mut fixtures = Fixtures {
        scheduler: [32, 128].map(scheduler_fixture).into(),
        graphs,
        sqlite,
        lab,
        campaign,
        executable,
        api_url,
        token,
        _server: server,
        _scratch: scratch,
    };

    let names = [
        "ecar_schedule_32",
        "evidence_snapshot_64",
        "evolution_24",
        "inference_decision",
        "sqlite_append",
        "api_list_latency",
        "report_generation",
        "evidence_snapshot_256",
        "ecar_schedule_128",
        "evolution_96",
        "worker_lifecycle",
        "bounded_concurrency_4",
    ];
    let samples = names.map(|name| {
        if name == "worker_lifecycle" {
            40
        } else if name == "bounded_concurrency_4" {
            20
        } else {
            SAMPLES
        }
    });
    let mut observations = vec![Vec::new(); names.len()];
    let mut order_state = 0x7a31_0c9d_u64;
    let mut execution_order = Vec::with_capacity(SAMPLES);

    for _ in 0..WARMUP {
        for name in names {
            std::hint::black_box(run_case(name, &mut fixtures)?);
        }
    }
    let start = Instant::now();
    for round in 0..SAMPLES {
        let mut order: Vec<usize> = (0..names.len()).collect();
        for i in (1..order.len()).rev() {
            let j = (next(&mut order_state) as usize) % (i + 1);
            order.swap(i, j);
        }
        execution_order.push(
            order
                .iter()
                .filter(|&&i| round < samples[i])
                .map(|&i| names[i])
                .collect::<Vec<_>>(),
        );
        for i in order {
            if round < samples[i] {
                observations[i].push(run_case(names[i], &mut fixtures)?);
            }
        }
    }
    let suite_wall = start.elapsed().as_secs_f64();
    let metrics = names
        .into_iter()
        .enumerate()
        .map(|(i, name)| summarize(name, observations[i].clone(), samples[i], i as u64 + 1))
        .collect::<Vec<_>>();
    Ok(json!({
        "schema":"aether.native-performance/1",
        "scope":"development-only native component timing; no model inference or held-out ECAR",
        "configuration":{"warmup_iterations":WARMUP,"counterbalanced_order":"deterministic Fisher-Yates shuffle per round","order_seed":"7a310c9d","execution_order":execution_order,"concurrency_workers":WORKERS,"bootstrap_repetitions":1000,"uncertainty":"percentile 95% bootstrap interval for median","p99_rule":"reported only with at least 1000 samples"},
        "host":{"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"logical_cpus":thread::available_parallelism().ok().map(std::num::NonZero::get)},
        "build":{"profile":if cfg!(debug_assertions){"debug"}else{"release"},"rustc":Command::new("rustc").arg("--version").output().ok().map(|o|String::from_utf8_lossy(&o.stdout).trim().to_owned()),"binary_sha256":aether_runner::sha256(&fs::read(&fixtures.executable)?)},
        "suite_wall_seconds":suite_wall,
        "metrics":metrics,
        "cost_boundaries":{"inference_decision":"pure action-selection function only; no provider/model","worker_lifecycle":"one reviewed native process per sample","report_generation":"in-process Lab::report","api_list_latency":"loopback HTTP including response-body read","bounded_concurrency_4":"four concurrent deterministic discovery tasks; thread creation included"}
    }))
}

fn run_case(name: &str, f: &mut Fixtures) -> Result<f64> {
    let start = Instant::now();
    let units = match name {
        "ecar_schedule_32" | "ecar_schedule_128" => {
            let candidates = &f.scheduler[usize::from(name.ends_with("128"))];
            black_box(schedule_with_trace(
                &budget(),
                black_box(candidates),
                ScoreWeights::default(),
                ScheduleContext::default(),
            )?);
            1
        }
        "evidence_snapshot_64" | "evidence_snapshot_256" => {
            black_box(f.graphs[usize::from(name.ends_with("256"))].snapshot("perf-graph")?);
            1
        }
        "evolution_24" | "evolution_96" => {
            black_box(discover(19, if name.ends_with("96") { 96 } else { 24 })?);
            1
        }
        "inference_decision" => {
            for call in 0..10_000 {
                black_box(inference::decision_action(
                    inference::Strategy::Adaptive,
                    call % 3,
                ));
            }
            10_000
        }
        "sqlite_append" => {
            f.sqlite.record_decision(
                "perf-sqlite",
                json!({"operation":"durable decision append"}),
            )?;
            1
        }
        "api_list_latency" => {
            let response = ureq::get(&format!("{}/v1/campaigns", f.api_url))
                .header("Authorization", &format!("Bearer {}", f.token))
                .call()?;
            black_box(response.into_body().read_to_string()?);
            1
        }
        "report_generation" => {
            black_box(f.lab.report(&f.campaign)?);
            1
        }
        "worker_lifecycle" => {
            black_box(supervise(
                &f.executable,
                &Task::MaxSubarray {
                    seed: 19,
                    items: 32,
                },
                Duration::from_secs(2),
                &AtomicBool::new(false),
            )?);
            1
        }
        "bounded_concurrency_4" => {
            thread::scope(|scope| {
                let handles = (0..WORKERS)
                    .map(|i| scope.spawn(move || discover(19 + u32::from(i), 24).unwrap()))
                    .collect::<Vec<_>>();
                for handle in handles {
                    black_box(handle.join().expect("bounded worker thread"));
                }
            });
            u64::from(WORKERS)
        }
        _ => return Err(format!("unknown benchmark {name}").into()),
    };
    Ok(start.elapsed().as_secs_f64() * 1_000_000.0
        / if name == "bounded_concurrency_4" {
            1.0
        } else {
            units as f64
        })
}

fn summarize(name: &'static str, samples: Vec<f64>, iterations: usize, seed: u64) -> Metric {
    let p50 = quantile(&samples, 0.50);
    let ci = bootstrap_median_ci(&samples, seed);
    let total_seconds = samples.iter().sum::<f64>() / 1_000_000.0
        * if name == "inference_decision" {
            10_000.0
        } else {
            1.0
        };
    let operations = iterations as f64
        * if name == "inference_decision" {
            10_000.0
        } else if name == "bounded_concurrency_4" {
            f64::from(WORKERS)
        } else {
            1.0
        };
    Metric {
        name,
        warmup_iterations: WARMUP,
        measured_iterations: samples.len(),
        p50_us: p50,
        p95_us: quantile(&samples, 0.95),
        p99_us: (samples.len() >= 1_000).then(|| quantile(&samples, 0.99)),
        median_ci95_us: ci,
        throughput_ops_per_second: operations / total_seconds,
        samples_us: samples,
    }
}

fn quantile(values: &[f64], q: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[((q * sorted.len() as f64).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1)]
}

fn bootstrap_median_ci(values: &[f64], seed: u64) -> [f64; 2] {
    let mut state = seed ^ 0xa57e_09d1;
    let mut medians = Vec::with_capacity(1_000);
    let mut draw = vec![0.0; values.len()];
    for _ in 0..1_000 {
        for x in &mut draw {
            *x = values[(next(&mut state) as usize) % values.len()];
        }
        medians.push(quantile(&draw, 0.5));
    }
    [quantile(&medians, 0.025), quantile(&medians, 0.975)]
}

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn budget() -> Budget {
    Budget::with_quarter_reserve(ResourceVec {
        wall_ms: 20_000,
        model_tokens: 2_000,
        attempts: 12,
        ..ResourceVec::default()
    })
}

fn scheduler_fixture(count: usize) -> Vec<Action> {
    (0..count)
        .map(|i| Action {
            id: format!("perf-{i}"),
            kind: [
                ActionKind::Explore,
                ActionKind::Replay,
                ActionKind::Challenge,
            ][i % 3],
            estimated_cost: ResourceVec {
                wall_ms: 100 + i as u64,
                attempts: 1,
                ..ResourceVec::default()
            },
            expected_gain: (i % 7 + 1) as u16,
            uncertainty_reduction: (i % 5) as u16,
            claim_importance: (i % 11) as u16,
            eligible: i % 7 != 6,
        })
        .collect()
}

fn graph_fixture(claims: usize) -> Result<Store> {
    let mut store = Store::open_in_memory()?;
    let hash = "1".repeat(64);
    store.create_campaign(&NewCampaign {
        id: "perf-graph".into(),
        title: "Performance fixture".into(),
        objective: "Graph traversal and SQLite".into(),
        protocol_hash: hash.clone(),
        policy_hash: hash.clone(),
        evaluator_hash: hash,
        requires_replay: false,
        caps: BudgetUnits::default(),
    })?;
    for i in 0..claims {
        store.add_claim(&NewClaim {
            id: format!("perf-claim-{i}"),
            campaign_id: "perf-graph".into(),
            statement: "Public synthetic fixture".into(),
        })?;
        if i > 0 {
            store.add_dependency(&format!("perf-claim-{i}"), &format!("perf-claim-{}", i - 1))?;
        }
    }
    Ok(store)
}

fn sqlite_fixture(path: PathBuf) -> Result<Store> {
    let mut store = Store::open(path)?;
    let hash = "2".repeat(64);
    store.create_campaign(&NewCampaign {
        id: "perf-sqlite".into(),
        title: "Durable SQLite fixture".into(),
        objective: "Measure committed local decision writes".into(),
        protocol_hash: hash.clone(),
        policy_hash: hash.clone(),
        evaluator_hash: hash,
        requires_replay: false,
        caps: BudgetUnits::default(),
    })?;
    Ok(store)
}

fn wait_for_api(url: &str) -> Result<()> {
    let until = Instant::now() + Duration::from_secs(10);
    while Instant::now() < until {
        if ureq::get(&format!("{url}/health")).call().is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err("local API failed to start for performance suite".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribution_summary_is_deterministic_and_withholds_unsupported_p99() {
        let a = summarize("fixture", (1..=100).map(f64::from).collect(), 100, 7);
        let b = summarize("fixture", (1..=100).map(f64::from).collect(), 100, 7);
        assert_eq!(a.p50_us, 50.0);
        assert_eq!(a.p95_us, 95.0);
        assert_eq!(a.p99_us, None);
        assert_eq!(a.median_ci95_us, b.median_ci95_us);
        assert!(a.throughput_ops_per_second > 0.0);
    }
}
