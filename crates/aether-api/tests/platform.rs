use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader};
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
fn reviewed_campaign_persists_independent_replay_and_verified_report() {
    let work = Workspace::new();
    let report = work.success(&["demo", "first", "29"]);
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
    let digest = report["protocol_hash"].as_str().unwrap();
    fs::write(
        work.0.join("protocols").join(format!("{digest}.json")),
        b"{}",
    )
    .unwrap();
    assert!(!work.invoke(&["report", "first"]).status.success());
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
}
