use crate::lab::{Error, Lab, PlanRequest, valid_id};
use crate::literature::{LiteratureError, MetadataProvider};
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{
        IntoResponse, Response, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post},
};
use futures_util::stream;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::io::Read;
use std::path::{Path as FsPath, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

#[derive(Clone)]
struct App {
    lab: Lab,
    token: Arc<str>,
    assets: PathBuf,
    active: Arc<Mutex<BTreeMap<String, Arc<AtomicBool>>>>,
    shutdown: Arc<AtomicBool>,
}

struct ApiError(Error);

type ApiResult<T> = Result<T, ApiError>;

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self.0 {
            Error::Input(message) => (StatusCode::BAD_REQUEST, "invalid_request", message),
            Error::Missing => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".into(),
            ),
            Error::Conflict(message) => (StatusCode::CONFLICT, "state_conflict", message),
            Error::Integrity(message) => (StatusCode::CONFLICT, "integrity_failed", message),
            Error::Io(_) | Error::Json(_) | Error::Schedule(_) | Error::Storage(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "operation_failed",
                "operation failed; inspect the local campaign record".into(),
            ),
        };
        (status, Json(json!({"code": code, "error": message}))).into_response()
    }
}

pub async fn serve(lab: Lab, port: u16, token: String, assets: PathBuf) -> std::io::Result<()> {
    let app = App {
        lab,
        token: token.into(),
        assets,
        active: Arc::new(Mutex::new(BTreeMap::new())),
        shutdown: Arc::new(AtomicBool::new(false)),
    };
    let protected = Router::new()
        .route("/v1/runtime", get(runtime))
        .route("/v1/hardware", get(hardware))
        .route("/v1/campaigns", get(list).post(create))
        .route("/v1/campaigns/{id}", get(detail))
        .route("/v1/campaigns/{id}/runs", get(runs))
        .route("/v1/campaigns/{id}/claims", get(claims))
        .route("/v1/campaigns/{id}/approve", post(approve))
        .route("/v1/campaigns/{id}/start", post(start))
        .route("/v1/campaigns/{id}/cancel", post(cancel))
        .route("/v1/campaigns/{id}/events", get(events))
        .route("/v1/campaigns/{id}/stream", get(live))
        .route("/v1/campaigns/{id}/report", get(report))
        .route("/v1/campaigns/{id}/protocol", get(protocol))
        .route("/v1/artifacts/{sha256}", get(artifact))
        .route("/v1/research/archive", get(research_archive))
        .route("/v1/research/archive/{id}", get(research_artifact))
        .route("/v1/models", get(model_registry))
        .route(
            "/v1/models/compare/{baseline}/{candidate}",
            get(model_comparison),
        )
        .route("/v1/training", get(training_registry))
        .route("/v1/datasets", get(dataset_registry).post(dataset_build))
        .route("/v1/datasets/qualify", post(dataset_qualify))
        .route("/v1/datasets/{digest}", get(dataset_inspect))
        .route("/v1/benchmarks", post(benchmarks))
        .route("/v1/evolution", post(evolution))
        .route("/v1/literature", get(literature))
        .route("/v1/literature/work", get(literature_work))
        .route("/v1/knowledge", get(knowledge_search))
        .route("/v1/knowledge/{id}", get(knowledge_export))
        .route("/v1/knowledge/{id}/hypotheses", get(knowledge_hypotheses))
        .route("/v1/knowledge/{id}/designs", get(knowledge_designs))
        .route("/v1/literature/lookup", post(lookup_metadata))
        .route("/v1/proposals", post(proposals))
        .layer(DefaultBodyLimit::max(8192))
        .route_layer(middleware::from_fn_with_state(app.clone(), authorize));
    let shutdown = app.clone();
    let router = protected
        .route("/", get(index))
        .route(
            "/health",
            get(|| async { Json(json!({"status":"ok","scope":"loopback"})) }),
        )
        .route("/assets/{name}", get(asset))
        .with_state(app);
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    eprintln!("AETHER listening on http://{}", listener.local_addr()?);
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            shutdown.shutdown.store(true, Ordering::Release);
            if let Ok(active) = shutdown.active.lock() {
                for control in active.values() {
                    control.store(true, Ordering::Release);
                }
            }
        })
        .await
}

async fn authorize(
    State(app): State<App>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let secret = app.token.as_bytes();
    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.as_bytes().strip_prefix(b"Bearer "))
        .filter(|supplied| supplied.len() == secret.len())
        .is_some_and(|supplied| {
            supplied
                .iter()
                .zip(secret)
                .fold(0u8, |difference, (&left, &right)| {
                    difference | (left ^ right)
                })
                == 0
        });
    if !authorized {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            Json(json!({"code": "unauthorized", "error": "a local bearer token is required"})),
        )
            .into_response();
    }
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, Error> + Send + 'static,
) -> ApiResult<T> {
    Ok(tokio::task::spawn_blocking(f)
        .await
        .map_err(|_| Error::Storage("operation interrupted".into()))??)
}

#[derive(Default, Deserialize)]
struct Page {
    after: Option<String>,
    limit: Option<usize>,
}

async fn list(State(app): State<App>, Query(page): Query<Page>) -> ApiResult<Json<Value>> {
    blocking(move || {
        Ok(serde_json::to_value(app.lab.campaigns_page(
            page.after.as_deref(),
            page.limit.unwrap_or(128),
        )?)?)
    })
    .await
    .map(Json)
}

async fn create(
    State(app): State<App>,
    Json(plan): Json<PlanRequest>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let body = blocking(move || Ok(serde_json::to_value(app.lab.create(&plan)?)?)).await?;
    Ok((StatusCode::CREATED, Json(body)))
}

async fn detail(State(app): State<App>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    blocking(move || app.lab.snapshot(&id)).await.map(Json)
}

async fn runs(State(app): State<App>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    blocking(move || {
        let snapshot = app.lab.snapshot(&id)?;
        Ok(json!({"runs": snapshot["runs"], "attempts": snapshot["attempts"]}))
    })
    .await
    .map(Json)
}

async fn claims(State(app): State<App>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    blocking(move || {
        let snapshot = app.lab.snapshot(&id)?;
        Ok(json!({"claims": snapshot["claims"], "dependencies": snapshot["dependencies"], "evidence": snapshot["evidence"]}))
    }).await.map(Json)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Approval {
    protocol_hash: String,
}

async fn approve(
    State(app): State<App>,
    Path(id): Path<String>,
    Json(approval): Json<Approval>,
) -> ApiResult<Json<Value>> {
    blocking(move || {
        Ok(serde_json::to_value(
            app.lab.approve(&id, &approval.protocol_hash)?,
        )?)
    })
    .await
    .map(Json)
}

async fn start(
    State(app): State<App>,
    Path(id): Path<String>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let control = Arc::new(AtomicBool::new(false));
    if !valid_id(&id) {
        return Err(Error::Input("invalid campaign id".into()).into());
    }
    let probe = app.lab.clone();
    let probe_id = id.clone();
    let snapshot = blocking(move || probe.snapshot(&probe_id)).await?;
    let mut active = app
        .active
        .lock()
        .map_err(|_| Error::Storage("supervisor unavailable".into()))?;
    if app.shutdown.load(Ordering::Acquire) {
        return Err(Error::Conflict("service is shutting down".into()).into());
    }
    if active.contains_key(&id) {
        return Ok((
            StatusCode::ACCEPTED,
            Json(json!({"id": id, "status": "running"})),
        ));
    }
    if snapshot["campaign"]["state"] != "approved" {
        return Err(Error::Conflict("campaign must be approved before starting".into()).into());
    }
    if !active.is_empty() {
        return Err(Error::Conflict("local worker is busy".into()).into());
    }
    active.insert(id.clone(), control.clone());
    drop(active);
    let job_id = id.clone();
    tokio::task::spawn_blocking(move || {
        if let Err(error) = app.lab.run(&job_id, &control) {
            eprintln!("campaign {job_id}: {error}");
        }
        if let Ok(mut active) = app.active.lock() {
            active.remove(&job_id);
        }
    });
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id": id, "status": "accepted"})),
    ))
}

async fn cancel(
    State(app): State<App>,
    Path(id): Path<String>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let control = app
        .active
        .lock()
        .map_err(|_| Error::Storage("supervisor unavailable".into()))?
        .get(&id)
        .cloned();
    let body = if let Some(handle) = control {
        handle.store(true, Ordering::Release);
        json!({"id": id, "status": "cancellation_requested"})
    } else {
        blocking(move || Ok(serde_json::to_value(app.lab.cancel_idle(&id)?)?)).await?
    };
    Ok((StatusCode::ACCEPTED, Json(body)))
}

#[derive(Default, Deserialize)]
struct Cursor {
    #[serde(default)]
    after: u64,
}

async fn events(
    State(app): State<App>,
    Path(id): Path<String>,
    Query(cursor): Query<Cursor>,
) -> ApiResult<Json<Vec<Value>>> {
    blocking(move || app.lab.events(&id, cursor.after))
        .await
        .map(Json)
}

async fn live(
    State(app): State<App>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Query(cursor): Query<Cursor>,
) -> ApiResult<impl IntoResponse> {
    let after = headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(cursor.after);
    let probe = app.lab.clone();
    let probe_id = id.clone();
    let first_page = blocking(move || probe.events(&probe_id, after)).await?;
    let updates = stream::unfold(
        (app.lab, id, after, app.shutdown, first_page.into_iter()),
        |(lab, id, mut after, shutdown, mut pending)| async move {
            loop {
                if shutdown.load(Ordering::Acquire) {
                    return None;
                }
                if let Some(event) = pending.next() {
                    after = event["sequence"].as_u64()?;
                    let datum = Event::default()
                        .id(after.to_string())
                        .event("campaign")
                        .data(event.to_string());
                    return Some((
                        Ok::<_, Infallible>(datum),
                        (lab, id, after, shutdown, pending),
                    ));
                }
                let query = lab.clone();
                let campaign = id.clone();
                pending = blocking(move || query.events(&campaign, after))
                    .await
                    .ok()?
                    .into_iter();
                if pending.len() == 0 {
                    tokio::time::sleep(Duration::from_millis(450)).await;
                }
            }
        },
    );
    Ok(Sse::new(updates).keep_alive(KeepAlive::default()))
}

async fn report(State(app): State<App>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    blocking(move || app.lab.report(&id)).await.map(Json)
}

async fn protocol(State(app): State<App>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    blocking(move || app.lab.protocol(&id)).await.map(Json)
}

async fn artifact(State(app): State<App>, Path(hash): Path<String>) -> ApiResult<Response> {
    let data = blocking(move || app.lab.artifact(&hash)).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CONTENT_DISPOSITION, "attachment"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        data,
    )
        .into_response())
}

async fn model_registry(State(app): State<App>) -> ApiResult<Json<Value>> {
    blocking(move || {
        app.lab
            .model_registry()?
            .list()
            .map_err(|error| Error::Storage(error.to_string()))
    })
    .await
    .map(Json)
}

async fn model_comparison(
    State(app): State<App>,
    Path((baseline, candidate)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    blocking(move || {
        crate::models::compare(&app.lab, &baseline, &candidate)
            .map_err(|error| Error::Input(error.to_string()))
    })
    .await
    .map(Json)
}

async fn training_registry(State(app): State<App>) -> ApiResult<Json<Value>> {
    blocking(move || {
        crate::models::training::list(&app.lab).map_err(|error| Error::Storage(error.to_string()))
    })
    .await
    .map(Json)
}

async fn dataset_registry(State(app): State<App>) -> ApiResult<Json<Value>> {
    blocking(move || {
        crate::models::datasets::list(&app.lab).map_err(|error| Error::Storage(error.to_string()))
    })
    .await
    .map(Json)
}

async fn dataset_build(State(app): State<App>) -> ApiResult<Json<Value>> {
    blocking(move || {
        crate::models::datasets::build(&app.lab).map_err(|error| Error::Storage(error.to_string()))
    })
    .await
    .map(Json)
}

async fn dataset_qualify(
    State(app): State<App>,
    Json(request): Json<crate::models::datasets::Request>,
) -> ApiResult<Json<Value>> {
    blocking(move || {
        crate::models::datasets::qualify(&app.lab, request)
            .map_err(|error| Error::Input(error.to_string()))
    })
    .await
    .map(Json)
}

async fn dataset_inspect(
    State(app): State<App>,
    Path(digest): Path<String>,
) -> ApiResult<Json<Value>> {
    blocking(move || {
        crate::models::datasets::inspect(&app.lab, &digest)
            .map_err(|error| Error::Input(error.to_string()))
    })
    .await
    .map(Json)
}

async fn research_archive() -> ApiResult<Json<Value>> {
    let performance = embedded(include_str!(
        "../../../research/results/dae-native-performance.json"
    ))?;
    let inference_fixture = embedded(include_str!(
        "../../../research/results/dae-inference-fixture.json"
    ))?;
    let inference_local = embedded(include_str!(
        "../../../research/results/dae-inference-local-qwen.json"
    ))?;
    let reliability = embedded(include_str!(
        "../../../research/results/dae-ecar-reliability-ablation.json"
    ))?;
    let ecar_pilot = embedded(include_str!(
        "../../../research/results/athena-ecar-summary.json"
    ))?;
    let baseline = embedded(include_str!(
        "../../../research/results/dae-knapsack-development.json"
    ))?;
    let hybrid = embedded(include_str!(
        "../../../research/results/dae-hybrid-development.json"
    ))?;
    let artifacts = [
        (
            "dae-knapsack-development",
            "DAE-02/03 matched search",
            include_bytes!("../../../research/results/dae-knapsack-development.json").as_slice(),
        ),
        (
            "dae-hybrid-development",
            "DAE-04 adaptive hybrid",
            include_bytes!("../../../research/results/dae-hybrid-development.json").as_slice(),
        ),
        (
            "dae-ecar-reliability-ablation",
            "DAE-05B reliability ablation",
            include_bytes!("../../../research/results/dae-ecar-reliability-ablation.json")
                .as_slice(),
        ),
        (
            "athena-ecar-summary",
            "Athena ECAR pilot summary",
            include_bytes!("../../../research/results/athena-ecar-summary.json").as_slice(),
        ),
        (
            "dae-inference-fixture",
            "DAE-06 deterministic inference fixture",
            include_bytes!("../../../research/results/dae-inference-fixture.json").as_slice(),
        ),
        (
            "dae-inference-local-qwen",
            "DAE-06 local Qwen pilot",
            include_bytes!("../../../research/results/dae-inference-local-qwen.json").as_slice(),
        ),
        (
            "dae-native-performance",
            "DAE-07 native performance run",
            include_bytes!("../../../research/results/dae-native-performance.json").as_slice(),
        ),
    ];
    Ok(Json(json!({
        "schema":"aether.research-archive/1",
        "recorded_on":"2026-11-26",
        "performance":performance,
        "inference_fixture":inference_fixture,
        "inference_local":inference_local,
        "reliability_ablation":{
            "scope":reliability["scope"],
            "outcomes_sha256":reliability["outcomes_sha256"],
            "prior_study_trace_changes":reliability["prior_study_trace_changes"],
            "cells":reliability["cells"],
            "contrasts":reliability["contrasts"]
        },
        "ecar_pilot":ecar_pilot,
        "discovery":{"baseline":discovery_summary(&baseline),"hybrid":discovery_summary(&hybrid)},
        "artifacts":artifacts.map(|(id,title,bytes)| json!({
            "id":id,"title":title,"sha256":aether_runner::sha256(bytes),"size_bytes":bytes.len()
        }))
    })))
}

async fn research_artifact(Path(id): Path<String>) -> ApiResult<Response> {
    let bytes: &'static [u8] = match id.as_str() {
        "dae-knapsack-development" => {
            include_bytes!("../../../research/results/dae-knapsack-development.json")
        }
        "dae-hybrid-development" => {
            include_bytes!("../../../research/results/dae-hybrid-development.json")
        }
        "dae-ecar-reliability-ablation" => {
            include_bytes!("../../../research/results/dae-ecar-reliability-ablation.json")
        }
        "athena-ecar-summary" => {
            include_bytes!("../../../research/results/athena-ecar-summary.json")
        }
        "dae-inference-fixture" => {
            include_bytes!("../../../research/results/dae-inference-fixture.json")
        }
        "dae-inference-local-qwen" => {
            include_bytes!("../../../research/results/dae-inference-local-qwen.json")
        }
        "dae-native-performance" => {
            include_bytes!("../../../research/results/dae-native-performance.json")
        }
        _ => return Err(Error::Missing.into()),
    };
    let mut response = Body::from(bytes).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        header::HeaderValue::from_static("nosniff"),
    );
    let digest = aether_runner::sha256(bytes);
    response.headers_mut().insert(
        "x-content-sha256",
        digest.parse().expect("SHA-256 is a valid header value"),
    );
    Ok(response)
}

fn embedded(source: &str) -> Result<Value, Error> {
    Ok(serde_json::from_str(source)?)
}

fn discovery_summary(report: &Value) -> Value {
    let mut arms = BTreeMap::<String, (u64, u64, f64, u64, u64)>::new();
    if let Some(cases) = report["evidence"].as_array() {
        for record in cases
            .iter()
            .flat_map(|case| case["records"].as_array().into_iter().flatten())
        {
            let Some(arm) = record["arm"].as_str() else {
                continue;
            };
            let values = arms.entry(arm.to_owned()).or_default();
            values.0 += 1;
            values.1 += u64::from(record["optimality_gap"].as_u64() == Some(0));
            values.2 += record["optimality_gap"].as_f64().unwrap_or_default();
            values.3 += record["trial"]["candidate_evaluations"]
                .as_u64()
                .unwrap_or_default();
            values.4 += record["trial"]["feasible_evaluations"]
                .as_u64()
                .unwrap_or_default();
        }
    }
    let arms = arms
        .into_iter()
        .map(|(name, (n, optimal, gap, evaluations, feasible))| {
            json!({
                "arm":name,"trials":n,"optimal":optimal,"mean_optimality_gap":gap/n.max(1) as f64,
                "candidate_evaluations":evaluations,"feasible_evaluations":feasible
            })
        })
        .collect::<Vec<_>>();
    json!({"scope":report["scope"],"evidence_sha256":report["evidence_sha256"],"cases":report["evidence"].as_array().map_or(0,Vec::len),"arms":arms})
}

async fn benchmarks() -> ApiResult<Json<Value>> {
    blocking(move || crate::benchmark().map_err(|error| Error::Schedule(error.to_string())))
        .await
        .map(Json)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvolutionRequest {
    seed: u32,
    evaluations: u16,
}

async fn evolution(Json(request): Json<EvolutionRequest>) -> ApiResult<Json<Value>> {
    blocking(move || {
        let result = aether_engine::evolve(request.seed, request.evaluations)
            .map_err(|error| Error::Input(error.to_string()))?;
        Ok(serde_json::to_value(result)?)
    })
    .await
    .map(Json)
}

async fn runtime(State(app): State<App>) -> Json<Value> {
    let active = app.active.lock().map_or(0, |jobs| jobs.len());
    Json(json!({
        "os": std::env::consts::OS, "arch": std::env::consts::ARCH,
        "logical_cpus": std::thread::available_parallelism().ok().map(std::num::NonZero::get),
        "worker_limit": 1, "active_campaigns": active,
        "worker_profile": "reviewed native executable, no untrusted code",
        "network_by_default": false, "isolation": "trusted built-in worker only",
    }))
}

async fn hardware() -> ApiResult<Json<Value>> {
    blocking(|| Ok(serde_json::to_value(crate::hardware::gpu_snapshot())?))
        .await
        .map(Json)
}

#[derive(Default, Deserialize)]
struct LiteratureQuery {
    #[serde(default)]
    query: String,
}

async fn literature(
    State(app): State<App>,
    Query(query): Query<LiteratureQuery>,
) -> ApiResult<Json<Vec<Value>>> {
    blocking(move || {
        Ok(app.lab
            .citations(&query.query)?
            .into_iter()
            .map(|item| {
                json!({
                    "title": item.title, "doi": item.doi, "year": item.published_year,
                    "source": item.container_title, "url": item.source_url(),
                    "arxiv_id": item.arxiv_id, "openalex_id": item.openalex_id, "topics": item.topics,
                    "provenance": item.provenance,
                })
            })
            .collect())
    })
    .await
    .map(Json)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataQuery {
    query: String,
    #[serde(default)]
    provider: MetadataProvider,
    #[serde(default)]
    save: bool,
    page: Option<u16>,
}

#[derive(Deserialize)]
struct LiteratureWorkQuery {
    id: String,
}

#[derive(Deserialize)]
struct KnowledgeSearch {
    #[serde(default)]
    query: String,
    kind: Option<crate::literature::GraphKind>,
}

#[derive(Deserialize)]
struct KnowledgeTraversal {
    depth: Option<u8>,
}

fn graph_error(error: LiteratureError) -> Error {
    match error {
        LiteratureError::MissingGraphNode => Error::Missing,
        LiteratureError::InvalidGraph(reason) => Error::Input(reason.into()),
        _ => Error::Storage(error.to_string()),
    }
}

async fn knowledge_search(
    State(app): State<App>,
    Query(query): Query<KnowledgeSearch>,
) -> ApiResult<Json<Value>> {
    blocking(move || {
        app.lab
            .catalog()?
            .graph_search(&query.query, query.kind)
            .map_err(graph_error)
    })
    .await
    .map(Json)
}

async fn knowledge_export(
    State(app): State<App>,
    Path(id): Path<String>,
    Query(query): Query<KnowledgeTraversal>,
) -> ApiResult<Json<crate::literature::GraphExport>> {
    blocking(move || {
        app.lab
            .catalog()?
            .graph_export(&id, query.depth.unwrap_or(1))
            .map_err(graph_error)
    })
    .await
    .map(Json)
}

async fn knowledge_hypotheses(
    State(app): State<App>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    blocking(move || {
        app.lab
            .catalog()?
            .discover_hypotheses(&id)
            .map_err(graph_error)
    })
    .await
    .map(Json)
}

async fn knowledge_designs(
    State(app): State<App>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    blocking(move || {
        app.lab
            .catalog()?
            .design_experiments(&id)
            .map_err(graph_error)
    })
    .await
    .map(Json)
}

async fn literature_work(
    State(app): State<App>,
    Query(query): Query<LiteratureWorkQuery>,
) -> ApiResult<Json<crate::literature::ArchiveWork>> {
    blocking(move || {
        app.lab
            .catalog()?
            .archive_work(&query.id)
            .map_err(|error| match error {
                LiteratureError::MissingCitation => Error::Missing,
                LiteratureError::InvalidCitation => Error::Input("invalid paper identifier".into()),
                _ => Error::Storage(error.to_string()),
            })
    })
    .await
    .map(Json)
}

async fn lookup_metadata(
    State(app): State<App>,
    Json(query): Json<MetadataQuery>,
) -> ApiResult<Response> {
    if !query.provider.enabled() {
        return Err(Error::Conflict("external citation lookup is disabled".into()).into());
    }
    let paginated = query.page.is_some();
    let result = blocking(move || {
        Ok(app.lab.catalog()?.discover(
            query.provider,
            &query.query,
            query.page.unwrap_or_default(),
            query.save,
        ))
    })
    .await?;
    match result {
        Ok(page) if paginated => Ok(Json(page).into_response()),
        Ok(page) => Ok(Json(page.records).into_response()),
        Err(LiteratureError::RateLimited { retry_after_secs }) => Ok((
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after_secs.to_string())],
            Json(json!({"code": "metadata_rate_limited", "error": "retry the metadata lookup later"})),
        )
            .into_response()),
        Err(LiteratureError::InvalidQuery) => Err(Error::Input("invalid metadata query".into()).into()),
        Err(error @ (LiteratureError::AliasConflict | LiteratureError::ArchiveLimit | LiteratureError::CatalogLimit)) =>
            Err(Error::Conflict(error.to_string()).into()),
        Err(_) => Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"code": "metadata_unavailable", "error": "metadata provider unavailable"})),
        )
            .into_response()),
    }
}

async fn proposals(
    Json(request): Json<crate::planning::PlanningRequest>,
) -> ApiResult<Json<Value>> {
    blocking(move || {
        let result = crate::planning::propose(&crate::planning::MockModel, &request)
            .map_err(|error| Error::Input(error.to_string()))?;
        Ok(serde_json::to_value(result)?)
    })
    .await
    .map(Json)
}

async fn index(State(app): State<App>) -> Response {
    serve_file(app.assets.join("index.html"), "text/html; charset=utf-8").await
}

async fn asset(State(app): State<App>, Path(name): Path<String>) -> Response {
    if name.len() > 100
        || name.starts_with('.')
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-'))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let kind = match FsPath::new(&name)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("woff2") => "font/woff2",
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    serve_file(app.assets.join("assets").join(name), kind).await
}

async fn serve_file(path: PathBuf, kind: &'static str) -> Response {
    let outcome = blocking(move || {
        let metadata = std::fs::symlink_metadata(&path)?;
        if !metadata.is_file() || metadata.len() > 4_194_304 {
            return Err(Error::Missing);
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(4_194_305)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 4_194_304 {
            return Err(Error::Missing);
        }
        Ok(bytes)
    })
    .await;
    match outcome {
        Ok(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, kind)
            .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
            .header(header::CONTENT_SECURITY_POLICY, "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'")
            .body(Body::from(bytes))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        Err(_) if kind.starts_with("text/html") => (StatusCode::SERVICE_UNAVAILABLE, "Build the workspace with npm ci and npm run build in web, then restart from the repository root.").into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
