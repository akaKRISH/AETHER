#![forbid(unsafe_code)]

mod hardware;
mod http;
mod inference;
mod lab;
mod literature;
mod planning;

use aether_engine::{StudyConfig, simulate};
use aether_runner::{SortAlgorithm, Task, WORKER_FLAG, evaluate, execute_worker, supervise};
use lab::{Lab, PlanRequest, read_json_file};
use serde_json::json;
use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const HELP: &str = "\
AETHER local research platform

Commands
  aether-lab demo <id> [seed]       Run a reviewed synthetic sorting campaign
  aether-lab plan <plan.json>       Freeze an explicit research protocol
  aether-lab plan-objective <id> <request.json> [<provider> <model>]  Freeze a model-guided sorting draft
  aether-lab approve <id> <sha256>  Approve exactly the frozen protocol digest
  aether-lab run <id>               Run an approved campaign
  aether-lab report <id>            Inspect evidence and validate artifact digests
  aether-lab list                   List local campaigns
  aether-lab benchmark [study.json] Execute the matched synthetic A–E simulation
  aether-lab benchmark-native       Measure bounded native sorting variants
  aether-lab evolve <seed> <evaluation-slots>  Compare four bounded knapsack search policies
  aether-lab discover <seed> <evaluation-slots>  Store and report one bounded knapsack search
  aether-lab inference-study <fixture|ollama>   Compare bounded inference strategies on development tasks
  aether-lab hardware               Read local NVIDIA driver telemetry, when available
  aether-lab propose <request.json> Produce deterministic reviewed-task proposals
  aether-lab propose-local <request.json> <model>  Use an existing local Ollama model
  aether-lab propose-provider <request.json> <provider> <model>  Plan with explicit provider provenance
  aether-lab bibliography [query]   Search the local citation catalog
  aether-lab import-citations <json> Import bounded DOI citation metadata
  aether-lab recover <id>           Reconcile an interrupted campaign
  aether-lab serve [port]           Start authenticated loopback API and built workspace

AETHER_DATA_DIR sets the local state directory (default .aether/lab).
AETHER_TOKEN is required to serve. Use a randomly generated token of 32–128 bytes.
";

fn main() -> ExitCode {
    let args = std::env::args_os()
        .skip(1)
        .map(|arg| {
            arg.into_string()
                .map_err(|_| "arguments must be valid Unicode")
        })
        .collect::<std::result::Result<Vec<_>, _>>();
    let result = match args {
        Ok(args) => execute(&args),
        Err(message) => Err(message.into()),
    };
    if let Err(error) = result {
        eprintln!("aether-lab: {error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn execute(args: &[String]) -> Result<()> {
    match args {
        [command, request] if command == WORKER_FLAG => {
            #[cfg(windows)]
            aether_runner::wait_for_admission()?;
            let result = execute_worker(request)?;
            io::stdout().lock().write_all(&result)?;
            return Ok(());
        }
        [] => {
            print!("{HELP}");
            return Ok(());
        }
        [command] if matches!(command.as_str(), "--help" | "-h" | "help") => {
            print!("{HELP}");
            return Ok(());
        }
        [command] if command == "benchmark" => {
            print_json(&benchmark()?)?;
            return Ok(());
        }
        [command, path] if command == "benchmark" => {
            print_json(&benchmark_config(&read_json_file(&PathBuf::from(path))?)?)?;
            return Ok(());
        }
        [command] if command == "benchmark-native" => {
            print_json(&benchmark_native()?)?;
            return Ok(());
        }
        [command] if command == "hardware" => return print_json(&hardware::gpu_snapshot()),
        [command, seed, evaluations] if command == "evolve" => {
            return print_json(&aether_engine::evolve(seed.parse()?, evaluations.parse()?)?);
        }
        [command, path] if command == "propose" => {
            let request = read_json_file(&PathBuf::from(path))?;
            return print_json(&planning::propose(&planning::MockModel, &request)?);
        }
        [command, path, model] if command == "propose-local" => {
            let request = read_json_file(&PathBuf::from(path))?;
            return print_json(&planning::propose(
                &planning::OllamaModel::new(model.clone())?,
                &request,
            )?);
        }
        [command, path, provider, model] if command == "propose-provider" => {
            let request = read_json_file(&PathBuf::from(path))?;
            return print_json(&planning::propose_with_provider(
                provider,
                model,
                &request,
                &AtomicBool::new(false),
            )?);
        }
        _ => {}
    }
    let root = std::env::var_os("AETHER_DATA_DIR")
        .map_or_else(|| PathBuf::from(".aether/lab"), PathBuf::from);
    let lab = Lab::open(root)?;
    match args {
        [command, provider] if command == "inference-study" => {
            let live = match provider.as_str() {
                "fixture" => false,
                "ollama" => true,
                _ => return Err("inference-study accepts fixture or local ollama".into()),
            };
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()?;
            let cancelled = std::sync::Arc::new(AtomicBool::new(false));
            let signal = cancelled.clone();
            runtime.spawn(async move {
                if tokio::signal::ctrl_c().await.is_ok() {
                    signal.store(true, std::sync::atomic::Ordering::Release);
                }
            });
            print_json(&inference::compare(&lab, live, &cancelled)?)
        }
        [command, seed, evaluations] if command == "discover" => {
            let report = aether_engine::discover(seed.parse()?, evaluations.parse()?)?;
            let digest = lab.record_discovery(&report)?;
            print_json(&json!({"artifact_sha256": digest, "report": report}))
        }
        [command, id] if command == "demo" => demo(&lab, id, 19),
        [command, id, seed] if command == "demo" => demo(&lab, id, seed.parse()?),
        [command, path] if command == "plan" => {
            print_json(&lab.create(&read_json_file(&PathBuf::from(path))?)?)
        }
        [command, id, path] if command == "plan-objective" => {
            plan_objective(&lab, id, path, "mock", "deterministic-v1")
        }
        [command, id, path, provider, model] if command == "plan-objective" => {
            plan_objective(&lab, id, path, provider, model)
        }
        [command, id, digest] if command == "approve" => print_json(&lab.approve(id, digest)?),
        [command, id] if command == "run" => print_json(&lab.run(id, &AtomicBool::new(false))?),
        [command, id] if command == "report" => print_json(&lab.report(id)?),
        [command] if command == "list" => print_json(&lab.campaigns()?),
        [command, id] if command == "recover" => {
            print_json(&json!({"expired_leases": lab.recover(id)?, "state": "failed"}))
        }
        [command] if command == "bibliography" => print_json(&lab.citations("")?),
        [command, query] if command == "bibliography" => print_json(&lab.citations(query)?),
        [command, path] if command == "import-citations" => {
            let citations = read_json_file(&PathBuf::from(path))?;
            print_json(&json!({"imported": lab.import_citations(citations)?}))
        }
        [command] if command == "serve" => serve(lab, 8080),
        [command, port] if command == "serve" => serve(lab, port.parse()?),
        _ => Err("unrecognized command; run aether-lab --help".into()),
    }
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    let mut output = io::stdout().lock();
    serde_json::to_writer_pretty(&mut output, value)?;
    output.write_all(b"\n")?;
    Ok(())
}

fn plan_objective(lab: &Lab, id: &str, path: &str, provider: &str, model: &str) -> Result<()> {
    let request: planning::ObjectiveRequest = read_json_file(&PathBuf::from(path))?;
    let (request, citations) = request.into_parts();
    let digest = request.digest()?;
    let _draft = lab.lock_objective_draft(id)?;
    let sources = lab.freeze_citations(&citations)?;
    let (campaign, planning) =
        match lab.existing_objective_draft(id, &digest, provider, model, &sources)? {
            Some(existing) => existing,
            None => {
                let advice = planning::propose_with_provider(
                    provider,
                    model,
                    &request,
                    &AtomicBool::new(false),
                )?;
                lab.create_objective_draft(id, request, &advice, sources)?
            }
        };
    print_json(&json!({
        "campaign": campaign,
        "planning": planning,
        "candidate_selection": "first eligible proposal from the selected provider",
    }))
}

fn demo(lab: &Lab, id: &str, seed: u32) -> Result<()> {
    let campaign = lab.create(&PlanRequest::example(id.to_owned(), seed))?;
    lab.approve(id, &campaign.spec.protocol_hash)?;
    let report = lab.run(id, &AtomicBool::new(false))?;
    print_json(&report)
}

fn benchmark() -> Result<serde_json::Value> {
    benchmark_config(&StudyConfig::default())
}

fn benchmark_config(config: &StudyConfig) -> Result<serde_json::Value> {
    let started = Instant::now();
    let report = simulate(config)?;
    Ok(json!({
        "scope": "matched synthetic scheduler simulation",
        "measurement": { "simulation_wall_us": started.elapsed().as_micros() as u64 },
        "report": report,
    }))
}

fn benchmark_native() -> Result<serde_json::Value> {
    let executable = std::env::current_exe()?;
    let cancelled = AtomicBool::new(false);
    let mut samples = Vec::new();
    for seed in [11, 19, 281] {
        for algorithm in [
            SortAlgorithm::Standard,
            SortAlgorithm::Insertion,
            SortAlgorithm::Selection,
        ] {
            let task = Task::Sort {
                seed,
                items: 128,
                algorithm,
            };
            let sample = supervise(&executable, &task, Duration::from_secs(2), &cancelled)?;
            let (correct, total) = evaluate(&task, &sample.output)?;
            samples.push(json!({
                "seed": seed, "algorithm": algorithm, "correct": correct, "total": total,
                "wall_ms": sample.wall_ms, "output_bytes": sample.output_bytes,
                "artifact_sha256": sample.artifact_sha256,
            }));
        }
    }
    Ok(json!({
        "scope": "development timing including native worker process startup and output capture",
        "versions": {"application": env!("CARGO_PKG_VERSION"), "benchmark": "aether.native-sort/1",
                     "binary_sha256": aether_runner::sha256(&std::fs::read(executable)?)},
        "host": {"os": std::env::consts::OS, "arch": std::env::consts::ARCH,
                 "logical_cpus": std::thread::available_parallelism().ok().map(std::num::NonZero::get)},
        "samples": samples,
        "interpretation": "Development timings only. This small measurement is insufficient to rank algorithms scientifically."
    }))
}

fn serve(lab: Lab, port: u16) -> Result<()> {
    let token = std::env::var("AETHER_TOKEN").map_err(|_| "set AETHER_TOKEN before serving")?;
    if !(32..=128).contains(&token.len()) || !token.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err("AETHER_TOKEN must contain 32–128 printable non-space ASCII characters".into());
    }
    let assets =
        std::env::var_os("AETHER_WEB_DIR").map_or_else(|| PathBuf::from("web/dist"), PathBuf::from);
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?
        .block_on(http::serve(lab, port, token, assets))?;
    Ok(())
}
