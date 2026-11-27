#![forbid(unsafe_code)]

mod hardware;
mod http;
mod inference;
mod lab;
mod literature;
mod models;
mod performance;
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
  aether-lab perf-lab               Run the bounded native performance suite
  aether-lab evolve <seed> <evaluation-slots>  Compare four bounded knapsack search policies
  aether-lab discover <seed> <evaluation-slots>  Store and report one bounded knapsack search
  aether-lab inference-study <fixture|ollama>   Compare bounded inference strategies on development tasks
  aether-lab models-discover [ollama-model-dir]  Inspect the local Ollama inventory
  aether-lab models-inspect <file> [sha256]     Inspect and hash a local GGUF or SafeTensors file
  aether-lab models-list                        List the saved model registry and evaluation reports
  aether-lab models-evaluate <fixture|fixture-regression|ollama>  Run the bounded development suite
  aether-lab models-compare <baseline-sha> <candidate-sha>       Compare two saved reports
  aether-lab training-propose <request.json>  Validate inputs and freeze an unapproved training proposal
  aether-lab training-fixture <lora|qlora>     Freeze a public synthetic proposal
  aether-lab training-rehearse <manifest-sha> <approval-sha> [cancel-before-step]  Exercise synthetic checkpoints
  aether-lab training-list                    Show saved proposals and rehearsal receipts
  aether-lab dataset-build                    Qualify the bundled public development corpus
  aether-lab dataset-qualify <request.json>    Validate public recipe and answer candidates
  aether-lab dataset-list                     List saved dataset manifests
  aether-lab dataset-inspect <manifest-sha>     Inspect a pinned dataset manifest
  aether-lab dataset-export <manifest-sha> <part>  Write a verified export or rejection report
  aether-lab skills-list                      List built-in and imported declarative skills
  aether-lab skill-inspect <skill-sha>         Inspect a pinned immutable skill version
  aether-lab skill-import <manifest.json>      Import an inert, untrusted skill manifest
  aether-lab skill-review <skill-sha>          Record local review of an imported declaration
  aether-lab skill-approve <skill-sha> <caps>  Approve exact declared capabilities (comma-separated)
  aether-lab skill-revoke <skill-sha>          Revoke a local skill approval
  aether-lab skill-invoke <skill-sha> <request.json>  Record preflight only; never executes skill instructions
  aether-lab skill-execute <request.json>       Execute one approved registered native operation
  aether-lab skill-compose <request.json>      Validate pinned dependencies and aggregate budgets
  aether-lab skills-invocations                List deterministic preflight records
  aether-lab skills-executions                 List integrity-checked native execution receipts
  aether-lab workflows-list                    List immutable workflow definitions
  aether-lab workflow-inspect <workflow-sha>    Inspect a pinned workflow definition
  aether-lab workflow-approve <workflow-sha>    Approve an exact workflow and skill closure
  aether-lab workflow-revoke <workflow-sha>     Revoke local workflow consent
  aether-lab workflow-execute <request.json>    Execute sequentially through the APO-09 kernel
  aether-lab workflows-executions               List integrity-checked workflow receipts
  aether-lab workflow-benchmark                  Run the two-process APO-10 fixture evaluation
  aether-lab workflow-benchmarks                 Inspect benchmark specification and reports
  aether-lab workflow-benchmark-inspect <sha256>  Inspect one content-addressed report
  aether-lab hardware               Read local NVIDIA driver telemetry, when available
  aether-lab propose <request.json> Produce deterministic reviewed-task proposals
  aether-lab propose-local <request.json> <model>  Use an existing local Ollama model
  aether-lab propose-provider <request.json> <provider> <model>  Plan with explicit provider provenance
  aether-lab bibliography [query]   Search the local citation catalog
  aether-lab import-citations <json> Import bounded DOI citation metadata
  aether-lab literature-discover <crossref|openalex|arxiv> <query> <page>  Cache and save one page
  aether-lab literature-work <paper-id>  Inspect metadata versions and extracted passages
  aether-lab literature-ingest <manifest.json> <full-text.txt>  Archive approved local UTF-8 text
  aether-lab literature-text <paper-id> <document-id>  Read exact archived source text
  aether-lab graph-apply <batch.json>  Record proposed entities and relationships
  aether-lab graph-paper <paper-id>    Index a saved paper and its extracted passages
  aether-lab graph-report <paper-id> <document-id> <start-byte> <end-byte>  Mark a source span as a reported claim
  aether-lab graph-campaign <id>      Capture trusted campaign records in the research graph
  aether-lab graph-export <node-id> [depth]  Export a bounded graph and its digest
  aether-lab hypothesis-index               Index the pinned DAE-04 observations
  aether-lab hypothesis-discover <node-id>  Rank advisory hypotheses for a bounded graph slice
  aether-lab design-experiments <node-id>    Export unapproved experimental designs
  aether-lab design-validate <draft.json>    Check a draft's digest and current provenance
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
        [command] if command == "--workflow-benchmark-fixture-runner" => {
            print_json(&models::workflow_evaluation::worker()?)?;
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
        [command] if command == "perf-lab" => {
            print_json(&performance::run()?)?;
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
        [command, provider]
            if matches!(command.as_str(), "inference-study" | "models-evaluate") =>
        {
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
            if command == "models-evaluate" {
                print_json(&models::evaluate(&lab, provider, &cancelled)?)
            } else {
                let live = match provider.as_str() {
                    "fixture" => false,
                    "ollama" => true,
                    _ => return Err("inference-study accepts fixture or local ollama".into()),
                };
                print_json(&inference::compare(&lab, live, &cancelled)?)
            }
        }
        [command] if command == "models-discover" => {
            print_json(&lab.model_registry()?.discover(&models::local_root()?)?)
        }
        [command, root] if command == "models-discover" => {
            print_json(&lab.model_registry()?.discover(&PathBuf::from(root))?)
        }
        [command, file] if command == "models-inspect" => {
            print_json(&lab.model_registry()?.inspect(&PathBuf::from(file), None)?)
        }
        [command, file, pin] if command == "models-inspect" => print_json(
            &lab.model_registry()?
                .inspect(&PathBuf::from(file), Some(pin))?,
        ),
        [command] if command == "models-list" => print_json(&lab.model_registry()?.list()?),
        [command, baseline, candidate] if command == "models-compare" => {
            print_json(&models::compare(&lab, baseline, candidate)?)
        }
        [command, path] if command == "training-propose" => {
            print_json(&models::training::propose(&lab, &PathBuf::from(path))?)
        }
        [command, method] if command == "training-fixture" => {
            print_json(&models::training::fixture(&lab, method)?)
        }
        [command, manifest, approval, rest @ ..]
            if command == "training-rehearse" && rest.len() <= 1 =>
        {
            let cancel = rest.first().map(|n| n.parse()).transpose()?;
            print_json(&models::training::rehearse(
                &lab, manifest, approval, cancel,
            )?)
        }
        [command] if command == "training-list" => print_json(&models::training::list(&lab)?),
        [command] if command == "dataset-build" => print_json(&models::datasets::build(&lab)?),
        [command, path] if command == "dataset-qualify" => {
            print_json(&models::datasets::qualify_file(&lab, &PathBuf::from(path))?)
        }
        [command] if command == "dataset-list" => print_json(&models::datasets::list(&lab)?),
        [command, pin] if command == "dataset-inspect" => {
            print_json(&models::datasets::inspect(&lab, pin)?)
        }
        [command, pin, part] if command == "dataset-export" => {
            io::stdout()
                .lock()
                .write_all(&models::datasets::export(&lab, pin, part)?)?;
            Ok(())
        }
        [command] if command == "skills-list" => print_json(&models::skills::list(&lab)?),
        [command, pin] if command == "skill-inspect" => {
            print_json(&models::skills::inspect(&lab, pin)?)
        }
        [command, path] if command == "skill-import" => {
            print_json(&models::skills::import(&lab, &PathBuf::from(path))?)
        }
        [command, pin] if command == "skill-review" => {
            print_json(&models::skills::review(&lab, pin)?)
        }
        [command, pin, caps] if command == "skill-approve" => print_json(&models::skills::approve(
            &lab,
            pin,
            models::skills::parse_capabilities(caps)?,
        )?),
        [command, pin] if command == "skill-revoke" => {
            print_json(&models::skills::revoke(&lab, pin)?)
        }
        [command, pin, path] if command == "skill-invoke" => {
            let request = read_json_file(&PathBuf::from(path))?;
            print_json(&models::skills::invoke(&lab, request, "local_cli")?)
        }
        [command, path] if command == "skill-compose" => {
            let request = read_json_file(&PathBuf::from(path))?;
            print_json(&models::skills::compose(&lab, request)?)
        }
        [command, path] if command == "skill-execute" => {
            let request = read_json_file(&PathBuf::from(path))?;
            print_json(&models::skills::execute(
                &lab,
                request,
                &AtomicBool::new(false),
            )?)
        }
        [command] if command == "skills-invocations" => {
            print_json(&models::skills::invocations(&lab)?)
        }
        [command] if command == "skills-executions" => {
            print_json(&models::skills::executions(&lab)?)
        }
        [command] if command == "workflows-list" => print_json(&models::workflows::list(&lab)?),
        [command, pin] if command == "workflow-inspect" => {
            print_json(&models::workflows::inspect(&lab, pin)?)
        }
        [command, pin] if command == "workflow-approve" => {
            print_json(&models::workflows::approve(&lab, pin)?)
        }
        [command, pin] if command == "workflow-revoke" => {
            print_json(&models::workflows::revoke(&lab, pin)?)
        }
        [command, path] if command == "workflow-execute" => {
            let request = serde_json::from_value(read_json_file(&PathBuf::from(path))?)?;
            print_json(&models::workflows::execute(
                &lab,
                request,
                &AtomicBool::new(false),
            )?)
        }
        [command] if command == "workflows-executions" => {
            print_json(&models::workflows::executions(&lab)?)
        }
        [command] if command == "workflow-benchmark" => {
            print_json(&models::workflow_evaluation::run(&lab)?)
        }
        [command] if command == "workflow-benchmarks" => {
            print_json(&models::workflow_evaluation::list(&lab)?)
        }
        [command, digest] if command == "workflow-benchmark-inspect" => {
            print_json(&models::workflow_evaluation::inspect(&lab, digest)?)
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
        [command, provider, query, page] if command == "literature-discover" => {
            use literature::MetadataProvider::{Arxiv, Crossref, OpenAlex};
            let provider = match provider.as_str() {
                "crossref" => Crossref,
                "openalex" => OpenAlex,
                "arxiv" => Arxiv,
                _ => return Err("unknown literature provider".into()),
            };
            if !provider.enabled() {
                return Err("external citation lookup is disabled".into());
            }
            print_json(
                &lab.catalog()?
                    .discover(provider, query, page.parse()?, true)?,
            )
        }
        [command, id] if command == "literature-work" => {
            print_json(&lab.catalog()?.archive_work(id)?)
        }
        [command, path, text] if command == "literature-ingest" => {
            let manifest: literature::TextManifest = read_json_file(&PathBuf::from(path))?;
            print_json(&lab.catalog()?.ingest_text(manifest, &PathBuf::from(text))?)
        }
        [command, id, digest] if command == "literature-text" => {
            print_json(&lab.catalog()?.source_text(id, digest)?)
        }
        [command, file] if command == "graph-apply" => {
            let batch: literature::GraphBatch = read_json_file(&PathBuf::from(file))?;
            print_json(&lab.catalog()?.graph_apply(batch)?)
        }
        [command, id] if command == "graph-paper" => {
            print_json(&lab.catalog()?.graph_index_paper(id)?)
        }
        [command, paper, document, start, end] if command == "graph-report" => print_json(
            &lab.catalog()?
                .graph_report_claim(paper, document, start.parse()?, end.parse()?)?,
        ),
        [command, id] if command == "graph-campaign" => print_json(&lab.graph_index_campaign(id)?),
        [command, id] if command == "graph-export" => {
            print_json(&lab.catalog()?.graph_export(id, 1)?)
        }
        [command, id, depth] if command == "graph-export" => {
            print_json(&lab.catalog()?.graph_export(id, depth.parse()?)?)
        }
        [command] if command == "hypothesis-index" => {
            print_json(&lab.catalog()?.index_hypothesis_archive()?)
        }
        [command, id] if command == "hypothesis-discover" => {
            print_json(&lab.catalog()?.discover_hypotheses(id)?)
        }
        [command, id] if command == "design-experiments" => {
            print_json(&lab.catalog()?.design_experiments(id)?)
        }
        [command, path] if command == "design-validate" => print_json(
            &lab.catalog()?
                .validate_experiment_design(read_json_file(&PathBuf::from(path))?)?,
        ),
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
