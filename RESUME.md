# Resume AETHER

## Checkpoint and phase

The current validated local checkpoint is `prometheus-windows-verified` on `main`. Recovery commit `41d3919` and foundation `c017c05` remain in history. The active phase is Prometheus qualification after the Windows integration release.

## Preserved implementation

- Frozen sorting and maximum-subarray pipelines, reviewed worker admission, journal snapshots, replay, and evidence export live in `crates/aether-api`, `crates/aether-runner`, and `crates/aether-store`.
- Model and literature adapters are in `crates/aether-api/src/planning` and `crates/aether-api/src/literature`. Four bounded knapsack arms and Criterion harnesses are in `crates/aether-engine` and `crates/aether-store`.
- The new browser controls, test harnesses, research dumps and delivery docs are in `web`, `tests`, `research/results`, and `docs`. The original sorting report SHA-256 remained `55ef0a797a07c7ce2e01538c3dd67e7e030621e14cd8b2bb19949cd8e1159b08`.

## Latest verification

Baseline passed 33 Rust and three frontend tests. The final Windows run passed 50 Rust and three frontend tests, format, strict Clippy, release build, frontend lint/type/build, browser smoke, advisory checks, and evidence validation. The release re-exported three prior reports byte for byte and completed a new objective-derived campaign with advice frozen under protocol v3. The hardware CLI observed the physical NVIDIA device. Crossref, OpenAlex and arXiv public probes returned 200. The recovery drill reconciled one interrupted lease. [Tests](docs/testing/TEST_RESULTS.md) · [Measurements](docs/research/BENCHMARK_RESULTS.md)

## Exact next action

Add exact local citation identity resolution to objective-derived planning and freeze selected sources with the hypothesis under a reviewed protocol. Reuse the typed records in `crates/aether-api/src/literature.rs` and the versioned freezing path in `crates/aether-api/src/lab.rs`. Preserve v1–v3 report compatibility and add a test for dangling identifiers. In parallel, qualify real inference only when a cloud-disabled authorized endpoint and sufficient memory are available.

## Remaining dependencies

Live model inference remains incomplete. Ollama 0.40.17 was reachable with cloud features enabled and no loaded model. AETHER's release policy gate rejected it before sending a prompt. GPU inference, stronger worker isolation and a held-out scientific evaluation remain separate qualification tracks. [Continuation](NEXT_ACTIONS.md) · [Provider status](docs/operations/MODEL_PROVIDERS.md)
