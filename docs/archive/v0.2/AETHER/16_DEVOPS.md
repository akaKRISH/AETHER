# 16 | DevOps, deployment and operations

## Repository proposal
`Cargo.toml` workspace; `crates/aether-core`, `aether-engine`, `aether-store`, `aether-runner`, `aether-cli` (under crates/); `workers/python/` optional; `contracts/`; `tests/{unit,integration,security,e2e}`; `taskpacks/{dev,heldout}` (heldout private evaluator during study); `docs/`; `research/{protocols,analysis}`; `infra/`; `web/` only at R2. Do not commit raw large artifacts, environment secrets or hidden test labels. Public release includes allowed reproduction artifacts with license notes.

## Local setup and CI
Use Git, rustup, a pinned stable toolchain and committed Cargo.lock; Python worker uses a separate locked virtual environment. Confirm WSL2/Linux and container support; test CPU path before CUDA. CI runs formatting, lint, type checks, unit tests, integration smoke, dependency audit and secret scan. Runner denial tests on isolated CI runner, not production. Pin CI action versions/commit digests when possible. Build signed/reproducible artifacts only after key/release process is designed. R0 ships CLI and example safe task pack, not a public server.

## Observability
Structured event fields: campaign_id, attempt_id, correlation_id, phase, duration_ms, error_class, CPU/GPU memory where available, tokens, provider cost and budget. Do not log sensitive payloads or secrets. Log rotation and output quotas. Metrics dashboard later shows queue, completion, failure, budget remaining and replay rates. Trigger local alerts on stalled lease, failed backup, quota violation or missing evidence. No background cloud monitoring is assumed.

## Remote compute
Free credits are conditional; prefer preemptible tasks with checkpoints, portable manifests and no personal data. Explicit user approval for provider, dataset, budget and network. Remote worker uses short-lived credentials, encrypted transport, least privilege and verified input digests. Avoid hosting the controller on public internet during R0/R1. Record hardware and cost; do not mix cloud and local timings without stratification.

## Recovery and release
Daily local metadata + referenced artifacts backup, verify hashes; quarterly or pre-release restore drill. Stop campaign before incompatible migration; snapshot database; test migration; roll back with backup if needed. Semantic versioning; schema versions independent. Release notes include known failures, limits, reproducibility instructions and model/data licenses. Proposed project license Apache-2.0, pending owner approval and dependency review.

## CI gates
Run cargo fmt --all -- --check; cargo clippy --workspace --all-targets --locked -- -D warnings; cargo test --workspace --locked; cargo audit (installed separately). Linux runner tests; Windows compile and smoke where supported; Python worker tests in separate job. Gate release on native fixture, schema compatibility and denial tests. Do not claim Windows sandbox parity before testing.
