use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const EXE: &str = env!("CARGO_BIN_EXE_aether-lab");
const TOKEN: &str = "local-test-token-for-aether-integration-only";
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "aether-lab-test-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn command(&self) -> Command {
        let mut command = Command::new(EXE);
        command
            .env("AETHER_DATA_DIR", &self.0)
            .env_remove("AETHER_ALLOW_EXTERNAL_MODELS")
            .env_remove("AETHER_ALLOW_CROSSREF");
        command
    }

    fn invoke(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn success(&self, args: &[&str]) -> Value {
        let output = self.invoke(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).expect("CLI JSON")
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn literature_ingestion_reopens_exact_source_and_never_creates_evidence() {
    let work = Workspace::new();
    let metadata = work.0.join("metadata.json");
    let manifest = work.0.join("source.json");
    let source = work.0.join("source.txt");
    let input = include_str!("fixtures/apollo-extraction.txt");
    let id = "10.9999/aether-apo01-fixture";
    fs::write(
        &metadata,
        json!([{"doi":id,"title":"Synthetic APOLLO test record"}]).to_string(),
    )
    .unwrap();
    fs::write(&source, input).unwrap();
    let mut rights = json!({
        "paper_id": "https://doi.org/10.9999/AETHER-APO01-FIXTURE",
        "source_url": "https://example.invalid/synthetic-apollo.txt",
        "license": "CC0-1.0",
        "rights_confirmed": false
    });
    fs::write(&manifest, rights.to_string()).unwrap();
    work.success(&["import-citations", metadata.to_str().unwrap()]);
    assert_eq!(
        work.success(&["literature-work", id])["text_status"],
        "unavailable"
    );
    let ingest = [
        "literature-ingest",
        manifest.to_str().unwrap(),
        source.to_str().unwrap(),
    ];
    assert!(!work.invoke(&ingest).status.success());
    rights["rights_confirmed"] = json!(true);
    fs::write(&manifest, rights.to_string()).unwrap();
    let archived = work.success(&ingest);
    assert_eq!(archived["status"], "extracted_unverified");
    assert_eq!(
        archived["source_sha256"],
        aether_store::artifact_sha256(input.as_bytes())
    );
    assert_eq!(archived["missing_sections"], json!([]));
    let passages = archived["passages"].as_array().unwrap();
    assert_eq!(passages.len(), 5);
    for passage in passages {
        let start = passage["start_byte"].as_u64().unwrap() as usize;
        let end = passage["end_byte"].as_u64().unwrap() as usize;
        assert_eq!(passage["quote"], &input[start..end]);
    }
    assert_eq!(work.success(&ingest), archived);
    assert_eq!(
        work.success(&["literature-text", id, archived["id"].as_str().unwrap()])["text"],
        input
    );
    assert_eq!(
        work.success(&["literature-work", id])["documents"],
        json!([archived])
    );
    assert_eq!(
        work.success(&["bibliography", "digests match"])
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(work.success(&["list"]), json!([]));
    let indexed = work.success(&["graph-paper", id]);
    let paper = indexed["paper"].as_str().unwrap();
    let graph = work.success(&["graph-export", paper, "1"]);
    assert_eq!(work.success(&["graph-export", paper, "1"]), graph);
    assert_eq!(graph["graph"]["nodes"].as_array().unwrap().len(), 6);
    let nodes = graph["graph"]["nodes"].as_array().unwrap();
    let injected = nodes
        .iter()
        .find(|node| {
            node["current"]["statement"]
                .as_str()
                .unwrap()
                .starts_with("Ignore earlier")
        })
        .unwrap();
    assert_eq!(injected["current"]["status"], "extracted");
    assert_eq!(
        injected["current"]["source"]["source_sha256"],
        archived["source_sha256"]
    );
    let injection_id = injected["id"].as_str().unwrap();
    let group = work.success(&["graph-export", injection_id, "1"]);
    assert_eq!(
        group["graph"]["source_group_peers"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    assert_eq!(group["graph"]["exact_duplicate_peers"], json!([]));

    let duplicate_id = "10.9999/aether-apo02-duplicate";
    fs::write(
        &metadata,
        json!([{"doi":duplicate_id,"title":"Synthetic duplicate source"}]).to_string(),
    )
    .unwrap();
    work.success(&["import-citations", metadata.to_str().unwrap()]);
    rights["paper_id"] = json!(duplicate_id);
    fs::write(&manifest, rights.to_string()).unwrap();
    work.success(&ingest);
    let group = work.success(&["graph-export", injection_id, "1"]);
    assert_eq!(
        group["graph"]["exact_duplicate_peers"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(work.success(&["list"]), json!([]));
    let catalog = rusqlite::Connection::open(work.0.join("literature.sqlite3")).unwrap();
    catalog
        .execute(
            "UPDATE literature_texts SET body = CAST('tampered' AS BLOB)",
            [],
        )
        .unwrap();
    assert!(
        !work
            .invoke(&["literature-text", id, archived["id"].as_str().unwrap()])
            .status
            .success()
    );
}

#[test]
fn graph_archives_two_rights_cleared_scientific_excerpts_offline() {
    let work = Workspace::new();
    let corpus: Vec<Value> = serde_json::from_str(include_str!(
        "../../../research/literature/apo02-reviewed-excerpts.json"
    ))
    .unwrap();
    let metadata = work.0.join("reviewed.json");
    let manifest = work.0.join("manifest.json");
    let source = work.0.join("excerpt.txt");
    fs::write(
        &metadata,
        serde_json::to_vec(
            &corpus
                .iter()
                .map(|row| &row["citation"])
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        work.success(&["import-citations", metadata.to_str().unwrap()])["imported"],
        2
    );
    for (row, kind) in corpus.iter().zip(["claim", "limitation"]) {
        let doi = row["citation"]["doi"].as_str().unwrap();
        let quote = row["quote"].as_str().unwrap();
        let body = format!("## {}\n{quote}\n", row["fixture_heading"].as_str().unwrap());
        fs::write(&source, &body).unwrap();
        fs::write(
            &manifest,
            json!({"paper_id": doi, "license": row["license"],
            "source_url": row["source_url"], "rights_confirmed": true})
            .to_string(),
        )
        .unwrap();
        let document = work.success(&[
            "literature-ingest",
            manifest.to_str().unwrap(),
            source.to_str().unwrap(),
        ]);
        assert_eq!(document["passages"].as_array().unwrap().len(), 1);
        let indexed = work.success(&["graph-paper", doi]);
        let root = indexed["paper"].as_str().unwrap();
        let graph = work.success(&["graph-export", root, "1"]);
        assert_eq!(graph["graph"]["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(graph["graph"]["edges"].as_array().unwrap().len(), 1);
        let passage = graph["graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["kind"] == kind)
            .unwrap();
        let current = &passage["current"];
        assert_eq!(current["status"], "extracted");
        assert_eq!(current["source"]["license"], "CC-BY-4.0");
        let start = current["source"]["start_byte"].as_u64().unwrap() as usize;
        let end = current["source"]["end_byte"].as_u64().unwrap() as usize;
        assert_eq!(&body[start..end], quote);
        assert_eq!(current["statement"], quote);
        assert_eq!(work.success(&["graph-export", root, "1"]), graph);
        if kind == "claim" {
            let document = document["id"].as_str().unwrap();
            assert!(
                !work
                    .invoke(&["graph-report", doi, document, "999", "1000"])
                    .status
                    .success()
            );
            let revised = work.success(&[
                "graph-report",
                doi,
                document,
                &start.to_string(),
                &end.to_string(),
            ]);
            let id = revised["claim"].as_str().unwrap();
            assert_eq!(id, passage["id"]);
            let graph = work.success(&["graph-export", id, "0"]);
            assert_eq!(graph["graph"]["root_history"][0]["status"], "extracted");
            assert_eq!(graph["graph"]["root_history"][1]["status"], "reported");
        }
    }
}

#[test]
fn hypothesis_archive_preserves_exact_pairs_across_processes() {
    let work = Workspace::new();
    let indexed = work.success(&["hypothesis-index"]);
    let id = indexed["root"].as_str().unwrap();
    let first = work.invoke(&["hypothesis-discover", id]);
    assert!(first.status.success());
    assert_eq!(
        first.stdout,
        work.invoke(&["hypothesis-discover", id]).stdout
    );
    assert_eq!(indexed, work.success(&["hypothesis-index"]));
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    let bytes = include_bytes!("../../../research/results/dae-hybrid-development.json");
    let artifact: Value = serde_json::from_slice(bytes).unwrap();
    let proposals = report["proposals"].as_array().unwrap();
    assert!(proposals.len() >= 3);
    for proposal in proposals {
        assert_eq!(proposal["status"], "unverified");
        for reference in proposal["supports"].as_array().unwrap() {
            let source = &reference["source"];
            assert_eq!(
                source["artifact_sha256"],
                aether_store::artifact_sha256(bytes)
            );
            let record = artifact
                .pointer(source["pointer"].as_str().unwrap())
                .unwrap();
            if reference["kind"] == "metric" {
                let comparison: Value =
                    serde_json::from_str(reference["statement"].as_str().unwrap()).unwrap();
                let records = record["records"].as_array().unwrap();
                for (arm, gap) in [
                    ("adaptive_hybrid", "treatment_gap"),
                    ("greedy", "control_gap"),
                ] {
                    assert!(records.iter().any(|row| row["arm"] == arm
                        && row["evaluation_budget"] == 8
                        && row["optimality_gap"] == comparison[gap]));
                }
            }
        }
    }
    assert_eq!(work.success(&["list"]), json!([]));
}

#[test]
fn discovery_persists_replayable_content_addressed_evidence() {
    let work = Workspace::new();
    let args = ["discover", "19", "8"];
    assert!(!work.invoke(&["discover", "19", "7"]).status.success());
    let result = work.success(&args);
    assert_eq!(result, work.success(&args));
    assert_eq!(result["report"]["scope"], "aether.knapsack-discovery/1");
    assert_eq!(result["report"]["stop"], "budget_exhausted");
    assert_eq!(
        result["report"]["search"]["archive"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    assert_eq!(work.success(&["list"]), json!([]));

    let digest = result["artifact_sha256"].as_str().unwrap();
    let db = work.0.join("laboratory.sqlite3");
    let store = aether_store::Store::open(&db).unwrap();
    let bytes = store.artifact(digest).unwrap().unwrap();
    assert_eq!(aether_store::artifact_sha256(&bytes), digest);
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).unwrap(),
        result["report"]
    );

    let db = rusqlite::Connection::open(&db).unwrap();
    assert!(
        db.execute(
            "UPDATE blob SET content = ? WHERE sha256 = ?",
            rusqlite::params![b"{}", digest],
        )
        .is_err()
    );
    let other = serde_json::to_vec(&aether_engine::discover(281, 8).unwrap()).unwrap();
    let other_digest = aether_store::artifact_sha256(&other);
    db.execute(
        "INSERT INTO blob VALUES (?, ?)",
        rusqlite::params![other_digest, b"{}"],
    )
    .unwrap();
    assert!(store.artifact(&other_digest).is_err());
    assert!(!work.invoke(&["discover", "281", "8"]).status.success());
}

#[test]
fn inference_study_persists_bounded_traces_and_independent_results() {
    let work = Workspace::new();
    let result = work.success(&["inference-study", "fixture"]);
    assert_eq!(result["complete"], true);
    let trials = result["trials"].as_array().unwrap();
    assert_eq!(trials.len(), 12);
    let db = aether_store::Store::open(work.0.join("laboratory.sqlite3")).unwrap();
    let mut saw_rejection = false;
    let mut saw_success = false;
    for trial in trials {
        let report = &trial["report"];
        let hash = trial["artifact_sha256"].as_str().unwrap();
        let bytes = db.artifact(hash).unwrap().unwrap();
        assert_eq!(aether_store::artifact_sha256(&bytes), hash);
        assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), *report);
        assert!(report["model_used"]["attempts"].as_u64().unwrap() <= 3);
        assert!(report["model_used"]["model_tokens"].as_u64().unwrap() <= 6_336);
        assert!(report["verifier_used"]["attempts"].as_u64().unwrap() <= 3);
        assert!(report["execution_used"]["attempts"].as_u64().unwrap() <= 1);
        for (i, candidate) in report["candidates"].as_array().unwrap().iter().enumerate() {
            assert!(
                candidate["parent"]
                    .as_u64()
                    .is_none_or(|parent| parent < i as u64)
            );
        }
        let steps = report["steps"].as_array().unwrap();
        assert_eq!(steps.last().unwrap()["action"], "STOP");
        saw_rejection |= steps.iter().any(|s| s["passed"] == false);
        saw_success |= steps.iter().any(|s| s["passed"] == true);
    }
    assert!(saw_rejection && saw_success);
    assert_eq!(work.success(&["list"]), json!([]));
}

#[test]
fn reviewed_campaign_persists_independent_replay_and_verified_report() {
    let work = Workspace::new();
    let report = work.success(&["demo", "first", "29"]);
    assert_eq!(report["protocol"]["version"], "aether.protocol/1");
    assert!(report["protocol"].get("planning").is_none());
    let evidence = &report["evidence"];
    assert_eq!(evidence["campaign"]["state"], "completed");
    assert!(
        evidence["claims"]
            .as_array()
            .unwrap()
            .iter()
            .any(|claim| claim["state"] == "verified")
    );
    assert_eq!(evidence["dependencies"].as_array().unwrap().len(), 1);
    assert_eq!(evidence["attempts"].as_array().unwrap().len(), 3);
    assert_eq!(evidence["replications"][0]["passed"], true);
    assert_ne!(
        evidence["replications"][0]["original_attempt_id"],
        evidence["replications"][0]["replay_attempt_id"]
    );
    let actions: Vec<_> = evidence["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|event| event["detail"]["trace"]["decision"]["detail"]["kind"].as_str())
        .collect();
    assert_eq!(actions, ["EXPLORE", "REPLAY", "CHALLENGE"]);
    assert_eq!(work.success(&["report", "first"]), report);
    let captured = work.success(&["graph-campaign", "first"]);
    let root = captured["campaign"].as_str().unwrap();
    let graph = work.success(&["graph-export", root, "2"]);
    assert!(
        graph["graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| {
                node["kind"] == "evidence" && node["current"]["status"] == "independently_verified"
            })
    );
    assert!(
        graph["graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|node| node["kind"] == "claim")
            .all(|node| node["current"]["status"] == "observed")
    );
    assert!(
        graph["graph"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|link| link["relation"] == "depends_on")
    );
    assert_eq!(work.success(&["graph-campaign", "first"]), captured);
    assert_eq!(work.success(&["graph-export", root, "2"]), graph);
    let digest = report["protocol_hash"].as_str().unwrap();
    fs::write(
        work.0.join("protocols").join(format!("{digest}.json")),
        b"{}",
    )
    .unwrap();
    assert!(!work.invoke(&["report", "first"]).status.success());
}

#[test]
fn frozen_optimization_preserves_negative_results_across_processes() {
    let work = Workspace::new();
    let plan_path = work.0.join("optimization.json");
    fs::write(
        &plan_path,
        include_bytes!("../../../taskpacks/example-max-subarray.json"),
    )
    .unwrap();
    let plan = work.success(&["plan", plan_path.to_str().unwrap()]);
    let id = plan["id"].as_str().unwrap();
    work.success(&["approve", id, plan["protocol_hash"].as_str().unwrap()]);
    let report = work.success(&["run", id]);
    assert_eq!(work.success(&["report", id]), report);

    let protocol = &report["protocol"];
    assert_eq!(protocol["version"], "aether.protocol/2");
    assert!(protocol.get("planning").is_none());
    let inputs = protocol["optimization_inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 2);
    assert_eq!(inputs[0].as_array().unwrap().len(), 128);
    assert_ne!(inputs[0], inputs[1]);

    let evidence = &report["evidence"];
    assert_eq!(evidence["campaign"]["state"], "completed");
    let runs = evidence["runs"].as_array().unwrap();
    let run = |action| runs.iter().find(|run| run["action"] == action).unwrap();
    assert_eq!(run("explore")["seed"], 19);
    assert_eq!(run("replay")["seed"], 19);
    assert_eq!(run("challenge")["seed"], 281);
    assert_eq!(run("explore")["input_hash"], run("replay")["input_hash"]);
    assert_ne!(run("explore")["input_hash"], run("challenge")["input_hash"]);
    assert_eq!(evidence["replications"][0]["passed"], true);
    assert_eq!(evidence["claims"][0]["state"], "rejected");
    assert_eq!(evidence["claims"][1]["state"], "verified");
    assert_eq!(evidence["contradictions"].as_array().unwrap().len(), 2);
    let attempts = evidence["attempts"].as_array().unwrap();
    assert_eq!(attempts.len(), 3);
    for attempt in attempts {
        let digest = attempt["manifest_sha256"].as_str().unwrap();
        let bytes: Vec<u8> =
            serde_json::from_value(report["blobs_by_sha256"][digest].clone()).unwrap();
        assert_eq!(aether_runner::sha256(&bytes), digest);
        let manifest: Value = serde_json::from_slice(&bytes).unwrap();
        let comparison = &manifest["optimization_comparison"];
        assert_eq!(comparison["candidate_sum"], comparison["reference_sum"]);
        assert_eq!(comparison["baseline_sum"], 127);
        assert_eq!(
            comparison["reference_sum"],
            if manifest["task"]["seed"] == 19 {
                623
            } else {
                1131
            }
        );
        assert_eq!(comparison["baseline_is_optimal"], false);
        assert_eq!(manifest["task"]["kind"], "max_subarray");
    }
}

#[test]
fn objective_planning_freezes_provenance_without_granting_execution() {
    let work = Workspace::new();
    let source = work.0.join("objective.json");
    let path = source.to_str().unwrap();
    let objective = "Compare sorting proposals with an independent reference.";
    fs::write(
        path,
        json!({"objective": objective, "seed": 19, "items": 32}).to_string(),
    )
    .unwrap();
    let denied = work.invoke(&["plan-objective", "hosted", path, "openai", "gpt-4o"]);
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("external-transfer and spend"));
    assert!(!work.invoke(&["report", "hosted"]).status.success());

    let drafted = work.success(&["plan-objective", "objective", path]);
    let protocol_hash = drafted["campaign"]["protocol_hash"].as_str().unwrap();
    assert_eq!(drafted["campaign"]["state"], "draft");
    assert_eq!(
        drafted,
        work.success(&[
            "plan-objective",
            "objective",
            path,
            "mock",
            "deterministic-v1"
        ])
    );
    assert!(!work.invoke(&["run", "objective"]).status.success());
    let frozen = fs::read(
        work.0
            .join("protocols")
            .join(format!("{protocol_hash}.json")),
    )
    .unwrap();
    assert_eq!(aether_runner::sha256(&frozen), protocol_hash);
    let protocol: Value = serde_json::from_slice(&frozen).unwrap();
    assert_eq!(protocol["version"], "aether.protocol/3");
    assert_eq!(protocol["planning"], drafted["planning"]);
    assert_eq!(protocol["plan"]["objective"], objective);
    assert_eq!(
        protocol["plan"]["algorithm"],
        drafted["planning"]["proposals"][0]["algorithm"]
    );
    assert_eq!(protocol["planning"]["provenance"]["provider"], "mock");

    fs::write(
        path,
        json!({"objective": objective, "seed": 281, "items": 32}).to_string(),
    )
    .unwrap();
    assert!(
        !work
            .invoke(&["plan-objective", "objective", path])
            .status
            .success()
    );
    assert_eq!(fs::read_dir(work.0.join("protocols")).unwrap().count(), 1);

    work.success(&["approve", "objective", protocol_hash]);
    let completed = work.success(&["run", "objective"]);
    assert_eq!(completed["evidence"]["campaign"]["state"], "completed");
    assert_eq!(completed["protocol"], protocol);
    assert_eq!(completed, work.success(&["report", "objective"]));
}

#[test]
fn cited_objective_binds_an_exact_local_record_before_approval() {
    let work = Workspace::new();
    let request = work.0.join("objective.json");
    let catalog = work.0.join("citations.json");
    fs::write(
        &request,
        json!({
            "objective":"Test a selected sorting method",
            "seed":19,"items":16,"citations":["10.9999/athena-test"]
        })
        .to_string(),
    )
    .unwrap();
    let args = ["plan-objective", "cited", request.to_str().unwrap()];
    assert!(!work.invoke(&args).status.success());
    assert_eq!(fs::read_dir(work.0.join("protocols")).unwrap().count(), 0);
    fs::write(
        &catalog,
        json!([{"doi":"10.9999/athena-test","title":"Synthetic catalog test record"}]).to_string(),
    )
    .unwrap();
    work.success(&["import-citations", catalog.to_str().unwrap()]);
    let draft = work.success(&args);
    assert_eq!(draft["campaign"]["state"], "draft");
    assert!(!work.invoke(&["run", "cited"]).status.success());
    assert_eq!(draft, work.success(&args));
    let report = work.success(&["report", "cited"]);
    assert_eq!(report["protocol"]["version"], "aether.protocol/4");
    assert_eq!(
        report["protocol"]["sources"][0]["id"],
        "10.9999/athena-test"
    );
    assert_eq!(
        report["protocol"]["sources"][0]["provenance"]["source"],
        "manual"
    );
}

#[test]
fn optimization_rejects_duplicate_inputs_and_preserves_the_first_frozen_protocol() {
    let work = Workspace::new();
    let plan_path = work.0.join("optimization.json");
    let path = plan_path.to_str().unwrap();
    let mut request = json!({
        "id":"one-element", "title":"One element", "objective":"Test the exact optimum edge case",
        "seed":19,"items":1,"attempts":5,"wall_ms":12000,
        "optimization":{"challenge_seed":19}
    });
    fs::write(path, request.to_string()).unwrap();
    assert!(!work.invoke(&["plan", path]).status.success());
    request["optimization"]["challenge_seed"] = json!(281);
    request["algorithm"] = json!("insertion");
    fs::write(path, request.to_string()).unwrap();
    assert!(!work.invoke(&["plan", path]).status.success());
    request.as_object_mut().unwrap().remove("algorithm");
    request["title"] = json!("Invalid\0title");
    fs::write(path, request.to_string()).unwrap();
    assert!(!work.invoke(&["plan", path]).status.success());
    assert_eq!(fs::read_dir(work.0.join("protocols")).unwrap().count(), 0);
    request["title"] = json!("One element");
    fs::write(path, request.to_string()).unwrap();
    let plan = work.success(&["plan", path]);
    assert_eq!(work.success(&["plan", path]), plan);
    request["title"] = json!("Different protocol, same immutable ID");
    fs::write(path, request.to_string()).unwrap();
    assert!(!work.invoke(&["plan", path]).status.success());
    assert_eq!(fs::read_dir(work.0.join("protocols")).unwrap().count(), 1);

    let id = plan["id"].as_str().unwrap();
    work.success(&["approve", id, plan["protocol_hash"].as_str().unwrap()]);
    let report = work.success(&["run", id]);
    let evidence = &report["evidence"];
    assert_eq!(evidence["claims"][0]["state"], "verified");
    assert_eq!(evidence["claims"][1]["state"], "verified");
    assert!(evidence["contradictions"].as_array().unwrap().is_empty());
}

#[test]
fn workspace_worker_lock_fences_another_process_before_dispatch() {
    let work = Workspace::new();
    let path = work.0.join("request.json");
    fs::write(
        &path,
        json!({
            "id":"slot", "title":"Worker slot", "objective":"Enforce the workspace worker limit",
            "seed":5,"items":8,"algorithm":"standard","attempts":1,"wall_ms":8000
        })
        .to_string(),
    )
    .unwrap();
    let plan = work.success(&["plan", path.to_str().unwrap()]);
    work.success(&["approve", "slot", plan["protocol_hash"].as_str().unwrap()]);
    let guard = fs::File::create(work.0.join("worker.lock")).unwrap();
    guard.try_lock().unwrap();
    let blocked = work.invoke(&["run", "slot"]);
    assert!(!blocked.status.success());
    assert!(String::from_utf8_lossy(&blocked.stderr).contains("workspace worker lock"));
    let report = work.success(&["report", "slot"]);
    assert_eq!(report["evidence"]["campaign"]["state"], "approved");
    assert_eq!(report["evidence"]["budget"]["used"]["attempts"], 0);
    drop(guard);
    work.success(&["run", "slot"]);
}

#[test]
fn one_attempt_budget_yields_inconclusive_claim() {
    let work = Workspace::new();
    let path = work.0.join("request.json");
    fs::write(
        &path,
        json!({
            "id":"bounded", "title":"Bounded attempt", "objective":"Check the one-attempt boundary",
            "seed":5,"items":64,"algorithm":"standard","attempts":1,"wall_ms":8000
        })
        .to_string(),
    )
    .unwrap();
    let plan = work.success(&["plan", path.to_str().unwrap()]);
    let digest = plan["protocol_hash"].as_str().unwrap();
    assert!(
        !work
            .invoke(&["approve", "bounded", &"0".repeat(64)])
            .status
            .success()
    );
    work.success(&["approve", "bounded", digest]);
    let report = work.success(&["run", "bounded"]);
    assert!(
        report["evidence"]["claims"]
            .as_array()
            .unwrap()
            .iter()
            .any(|claim| claim["state"] == "inconclusive")
    );
    assert_eq!(report["evidence"]["budget"]["used"]["attempts"], 1);
    assert!(
        report["evidence"]["replications"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn recovery_closes_a_stranded_campaign_without_erasing_evidence() {
    use aether_store::{CampaignState, Store};
    let work = Workspace::new();
    let path = work.0.join("interrupted-plan.json");
    fs::write(&path, json!({
        "id":"interrupted", "title":"Interrupted", "objective":"Recover after a supervisor exits",
        "seed":5,"items":64,"algorithm":"standard","attempts":5,"wall_ms":8000
    }).to_string()).unwrap();
    let plan = work.success(&["plan", path.to_str().unwrap()]);
    work.success(&[
        "approve",
        "interrupted",
        plan["protocol_hash"].as_str().unwrap(),
    ]);
    let mut store = Store::open(work.0.join("laboratory.sqlite3")).unwrap();
    store
        .transition_campaign("interrupted", CampaignState::Running)
        .unwrap();
    drop(store);
    assert_eq!(work.success(&["recover", "interrupted"])["state"], "failed");
    let report = work.success(&["report", "interrupted"]);
    assert_eq!(report["evidence"]["campaign"]["state"], "failed");
    assert!(
        report["evidence"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["detail"]["recovery"].as_str().is_some())
    );
}

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn experimental_designs_reproduce_and_cannot_enter_campaigns() {
    let work = Workspace::new();
    let indexed = work.success(&["hypothesis-index"]);
    let root = indexed["root"].as_str().unwrap();
    let export = ["design-experiments", root];
    let first = work.invoke(&export);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, work.invoke(&export).stdout);
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    let designs = report["designs"].as_array().unwrap();
    assert_eq!(designs.len(), 3);
    let path = work.0.join("draft.json");
    let file = path.to_str().unwrap();
    for draft in designs {
        fs::write(&path, draft.to_string()).unwrap();
        let receipt = work.success(&["design-validate", file]);
        assert_eq!(receipt["valid_review_draft"], true);
        assert_eq!(receipt["approval_status"], "unapproved");
        assert!(!work.invoke(&["plan", file]).status.success());
    }
    fs::write(&path, json!({
        "id": "design-boundary", "title": "Ordinary draft", "objective": "Check approval binding",
        "seed": 19, "items": 8, "algorithm": "insertion", "attempts": 5, "wall_ms": 12000
    }).to_string()).unwrap();
    let before = work.success(&["plan", file]);
    let digest = designs[0]["protocol_sha256"].as_str().unwrap();
    let rejected = work.invoke(&["approve", "design-boundary", digest]);
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("does not match frozen protocol"));
    assert_eq!(before, work.success(&["plan", file]));

    // Even a misindexed digest with matching bytes must fail campaign approval.
    let bytes = serde_json::to_vec(&designs[0]["protocol"]).unwrap();
    let mut spec: aether_store::NewCampaign = serde_json::from_value(before).unwrap();
    spec.id = "misindexed-design".into();
    spec.protocol_hash = aether_store::artifact_sha256(&bytes);
    fs::write(
        work.0
            .join("protocols")
            .join(format!("{}.json", spec.protocol_hash)),
        bytes,
    )
    .unwrap();
    let mut store = aether_store::Store::open(work.0.join("laboratory.sqlite3")).unwrap();
    store.create_campaign(&spec).unwrap();
    let rejected = work.invoke(&["approve", &spec.id, &spec.protocol_hash]);
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("approval requires a frozen campaign protocol")
    );
    assert_eq!(
        store.campaign(&spec.id).unwrap().unwrap().state,
        aether_store::CampaignState::Draft
    );
}

#[test]
fn model_lab_persists_local_metadata_and_reproduces_matched_regressions() {
    let work = Workspace::new();
    let path = work.0.join("test.safetensors");
    let header = br#"{"weight":{"dtype":"I8","shape":[2],"data_offsets":[0,2]}}"#;
    let bytes = (header.len() as u64)
        .to_le_bytes()
        .into_iter()
        .chain(header.iter().copied())
        .chain([0, 0])
        .collect::<Vec<_>>();
    fs::write(&path, &bytes).unwrap();
    let pin = aether_store::artifact_sha256(&bytes);
    let path = path.to_str().unwrap();
    assert!(
        !work
            .invoke(&["models-inspect", path, &"0".repeat(64)])
            .status
            .success()
    );
    let inspected = work.success(&["models-inspect", path, &pin]);
    assert_eq!(inspected["inspection"]["stored_elements"], 2);
    assert_eq!(inspected["integrity"], "expected_digest_matched");
    assert_eq!(work.success(&["models-list"])["entries"][0], inspected);

    let baseline = work.success(&["models-evaluate", "fixture"]);
    assert_eq!(baseline, work.success(&["models-evaluate", "fixture"]));
    assert_eq!(baseline["report"]["summary"]["passed"], 7);
    assert_eq!(baseline["report"]["summary"]["calibration_samples"], 5);
    let changed = work.success(&["models-evaluate", "fixture-regression"]);
    assert_eq!(changed["report"]["summary"]["passed"], 5);
    assert_eq!(changed["report"]["summary"]["brier_numerator"], 80_000_000);
    let left = baseline["artifact_sha256"].as_str().unwrap();
    let right = changed["artifact_sha256"].as_str().unwrap();
    let diff = work.success(&["models-compare", left, right]);
    let regressions = diff["paired"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["regression"] == true)
        .map(|r| r["case"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(regressions, ["calibration_hard", "safety"]);
    assert_eq!(
        work.success(&["models-list"])["reports"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        !work
            .invoke(&["models-compare", left, &pin])
            .status
            .success()
    );
}

#[test]
fn authenticated_loopback_api_runs_a_campaign_and_serves_evidence() {
    let work = Workspace::new();
    let mut server = Server(
        work.command()
            .args(["serve", "0"])
            .env("AETHER_TOKEN", TOKEN)
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let mut output = BufReader::new(server.0.stderr.take().unwrap());
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    let origin = line.trim().strip_prefix("AETHER listening on ").unwrap();
    assert!(origin.starts_with("http://127.0.0.1:"));
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(4)))
            .http_status_as_error(false)
            .build(),
    );
    let get = |path: &str| {
        agent
            .get(format!("{origin}{path}"))
            .header("Authorization", format!("Bearer {TOKEN}"))
            .call()
            .unwrap()
    };
    let post = |path: &str, body: Value| {
        agent
            .post(format!("{origin}{path}"))
            .header("Authorization", format!("Bearer {TOKEN}"))
            .send_json(body)
            .unwrap()
    };
    let unauthenticated = agent.get(format!("{origin}/v1/campaigns")).call().unwrap();
    assert_eq!(unauthenticated.status().as_u16(), 401);
    assert_eq!(
        agent
            .get(format!("{origin}/v1/models"))
            .call()
            .unwrap()
            .status()
            .as_u16(),
        401
    );
    assert_eq!(post("/v1/models", json!({})).status().as_u16(), 405);
    let model_report = work.success(&["models-evaluate", "fixture"]);
    let registry: Value = get("/v1/models").body_mut().read_json().unwrap();
    assert_eq!(
        registry["reports"][0]["artifact_sha256"],
        model_report["artifact_sha256"]
    );
    let indexed = work.success(&["hypothesis-index"]);
    let root = indexed["root"].as_str().unwrap();
    let designs_path = format!("/v1/knowledge/{root}/designs");
    assert_eq!(
        agent
            .get(format!("{origin}{designs_path}"))
            .call()
            .unwrap()
            .status()
            .as_u16(),
        401
    );
    let mut response = get(&designs_path);
    assert_eq!(response.status().as_u16(), 200);
    let designs: Value = response.body_mut().read_json().unwrap();
    assert_eq!(designs, work.success(&["design-experiments", root]));
    assert_eq!(post(&designs_path, json!({})).status().as_u16(), 405);
    assert_eq!(
        post("/v1/campaigns", json!({
            "protocol_sha256": designs["designs"][0]["protocol_sha256"], "approval_status": "unapproved"
        }))
            .status()
            .as_u16(),
        422
    );
    assert_eq!(
        agent
            .get(format!("{origin}/v1/research/archive"))
            .call()
            .unwrap()
            .status()
            .as_u16(),
        401
    );
    let archive: Value = get("/v1/research/archive").body_mut().read_json().unwrap();
    assert_eq!(archive["schema"], "aether.research-archive/1");
    assert_eq!(
        archive["performance"]["schema"],
        "aether.native-performance/1"
    );
    let artifact = get("/v1/research/archive/dae-native-performance");
    assert_eq!(artifact.status().as_u16(), 200);
    let digest = artifact
        .headers()
        .get("x-content-sha256")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let mut bytes = Vec::new();
    artifact
        .into_body()
        .into_reader()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(aether_runner::sha256(&bytes), digest);
    assert_eq!(
        digest,
        aether_runner::sha256(include_bytes!(
            "../../../research/results/dae-native-performance.json"
        ))
    );
    assert_eq!(get("/v1/research/archive/unknown").status().as_u16(), 404);
    assert_eq!(
        post("/v1/literature/lookup", json!({"query":"sorting"}))
            .status()
            .as_u16(),
        409
    );
    let mut created = post(
        "/v1/campaigns",
        json!({
            "id":"via-http", "title":"HTTP campaign", "objective":"Exercise the native loopback contract",
            "seed":9,"items":64,"algorithm":"selection","attempts":5,"wall_ms":12000
        }),
    );
    assert_eq!(created.status().as_u16(), 201);
    let campaign: Value = created.body_mut().read_json().unwrap();
    assert_eq!(
        post("/v1/campaigns/via-http/start", json!({}))
            .status()
            .as_u16(),
        409
    );
    assert_eq!(
        post(
            "/v1/campaigns/via-http/approve",
            json!({"protocol_hash": campaign["protocol_hash"]})
        )
        .status()
        .as_u16(),
        200
    );
    assert_eq!(
        post("/v1/campaigns/via-http/start", json!({}))
            .status()
            .as_u16(),
        202
    );
    let started = Instant::now();
    loop {
        let snapshot: Value = get("/v1/campaigns/via-http")
            .body_mut()
            .read_json()
            .unwrap();
        match snapshot["campaign"]["state"].as_str() {
            Some("completed") => {
                assert!(
                    snapshot["claims"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|claim| claim["state"] == "verified")
                );
                break;
            }
            Some("failed") => panic!("campaign failed: {snapshot}"),
            _ => {}
        }
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "campaign did not finish: {snapshot}"
        );
        std::thread::sleep(Duration::from_millis(40));
    }
    let report: Value = get("/v1/campaigns/via-http/report")
        .body_mut()
        .read_json()
        .unwrap();
    let hash = report["evidence"]["artifacts"][0]["sha256"]
        .as_str()
        .unwrap();
    let bytes = get(&format!("/v1/artifacts/{hash}"))
        .body_mut()
        .read_to_vec()
        .unwrap();
    assert_eq!(aether_runner::sha256(&bytes), hash);
    let items: Vec<Value> = get("/v1/campaigns/via-http/events?after=0")
        .body_mut()
        .read_json()
        .unwrap();
    assert!(items.len() > 10);
    let after = items[0]["sequence"].as_u64().unwrap();
    let expected = items[1]["sequence"].as_u64().unwrap();
    let stream_path = format!("{origin}/v1/campaigns/via-http/stream?after=0");
    assert_eq!(
        agent.get(&stream_path).call().unwrap().status().as_u16(),
        401
    );
    let mut stream_response = agent
        .get(stream_path)
        .header("Authorization", format!("Bearer {TOKEN}"))
        .header("Last-Event-ID", after.to_string())
        .call()
        .unwrap();
    assert_eq!(
        stream_response.headers().get("content-type").unwrap(),
        "text/event-stream"
    );
    let mut data = None;
    for line in BufReader::new(stream_response.body_mut().as_reader())
        .lines()
        .take(4)
    {
        let line = line.unwrap();
        if let Some(value) = line.strip_prefix("data: ") {
            data = Some(serde_json::from_str::<Value>(value).unwrap());
            break;
        }
    }
    assert_eq!(data.unwrap()["sequence"], expected);
    drop(stream_response);

    assert_eq!(
        post(
            "/v1/campaigns",
            json!({
                "id":"cancelled", "title":"Cancel draft", "objective":"Exercise repeated cancellation",
                "seed":9,"items":64,"algorithm":"selection","attempts":5,"wall_ms":12000
            })
        )
        .status()
        .as_u16(),
        201
    );
    for _ in 0..2 {
        assert_eq!(
            post("/v1/campaigns/cancelled/cancel", json!({}))
                .status()
                .as_u16(),
            202
        );
    }
    let cancelled: Value = get("/v1/campaigns/cancelled")
        .body_mut()
        .read_json()
        .unwrap();
    assert_eq!(cancelled["campaign"]["state"], "cancelled");
    assert_eq!(cancelled["attempts"], json!([]));
    let literature: Vec<Value> = get("/v1/literature").body_mut().read_json().unwrap();
    assert!(literature.iter().any(|item| {
        item["title"]
            .as_str()
            .unwrap_or_default()
            .contains("CAVEAT")
    }));
    let path = "/v1/literature/work?id=10.21203%2Frs.3.rs-11035327%2Fv1";
    assert_eq!(
        agent
            .get(format!("{origin}{path}"))
            .call()
            .unwrap()
            .status()
            .as_u16(),
        401
    );
    let detail: Value = get(path).body_mut().read_json().unwrap();
    assert_eq!(detail["text_status"], "unavailable");
    assert_eq!(detail["snapshots"].as_array().unwrap().len(), 1);
    let search = "/v1/knowledge?query=CAVEAT&kind=paper";
    assert_eq!(
        agent
            .get(format!("{origin}{search}"))
            .call()
            .unwrap()
            .status()
            .as_u16(),
        401
    );
    let entities: Value = get(search).body_mut().read_json().unwrap();
    let node = entities["nodes"][0]["id"].as_str().unwrap();
    let graph = format!("/v1/knowledge/{node}");
    let hypotheses = format!("{graph}/hypotheses");
    assert_eq!(
        agent
            .get(format!("{origin}{hypotheses}"))
            .call()
            .unwrap()
            .status()
            .as_u16(),
        401
    );
    let sparse: Value = get(&hypotheses).body_mut().read_json().unwrap();
    assert_eq!(sparse["schema"], "aether.hypotheses/1");
    assert_eq!(sparse["proposals"], json!([]));
    assert_eq!(
        agent
            .get(format!("{origin}{graph}"))
            .call()
            .unwrap()
            .status()
            .as_u16(),
        401
    );
    let exported: Value = get(&graph).body_mut().read_json().unwrap();
    assert_eq!(exported["graph"]["root"], node);
    assert_eq!(
        exported["graph"]["nodes"][0]["current"]["status"],
        "reported"
    );
    assert_eq!(get(&format!("{graph}?depth=4")).status().as_u16(), 400);
    assert_eq!(
        get("/v1/literature/work?id=10.9999%2Fabsent")
            .status()
            .as_u16(),
        404
    );
}
