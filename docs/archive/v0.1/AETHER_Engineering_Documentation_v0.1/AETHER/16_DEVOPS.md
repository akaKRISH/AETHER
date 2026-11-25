# 16 | DevOps, deployment and operations

## Repository proposal
`src/aether/{domain,protocol,scheduler,budget,runner,evaluator,ledger,report,adapters}`; `tests/{unit,integration,security,e2e}`; `taskpacks/{dev,heldout}` (heldout private evaluator during study); `docs/`; `research/{protocols,analysis}`; `infra/`; `web/` only at R2. Do not commit raw large artifacts, environment secrets or hidden test labels. Public release includes allowed reproduction artifacts with license notes.

## Local setup and CI
Use Git, Python virtual environment and locked dependencies. Confirm WSL2/Linux and container support; test CPU path before CUDA. CI runs formatting, lint, type checks, unit tests, integration smoke, dependency audit and secret scan. Runner denial tests on isolated CI runner, not production. Pin CI action versions/commit digests when possible. Build signed/reproducible artifacts only after key/release process is designed. R0 ships CLI and example safe task pack, not a public server.

## Observability
Structured event fields: campaign_id, attempt_id, correlation_id, phase, duration_ms, error_class, CPU/GPU memory where available, tokens, provider cost and budget. Do not log sensitive payloads or secrets. Log rotation and output quotas. Metrics dashboard later shows queue, completion, failure, budget remaining and replay rates. Trigger local alerts on stalled lease, failed backup, quota violation or missing evidence. No background cloud monitoring is assumed.

## Remote compute
Free credits are conditional; prefer preemptible tasks with checkpoints, portable manifests and no personal data. Explicit user approval for provider, dataset, budget and network. Remote worker uses short-lived credentials, encrypted transport, least privilege and verified input digests. Avoid hosting the controller on public internet during R0/R1. Record hardware and cost; do not mix cloud and local timings without stratification.

## Recovery and release
Daily local metadata + referenced artifacts backup, verify hashes; quarterly or pre-release restore drill. Stop campaign before incompatible migration; snapshot database; test migration; roll back with backup if needed. Semantic versioning; schema versions independent. Release notes include known failures, limits, reproducibility instructions and model/data licenses. Proposed project license Apache-2.0, pending owner approval and dependency review.
