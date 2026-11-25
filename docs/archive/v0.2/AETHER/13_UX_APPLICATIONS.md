# 13 | User experience and computer applications

## R0 CLI user journey
`aether init` creates project; `aether protocol validate task.json` previews config; `aether approve <hash>` freezes approval; `aether run <protocol-id>` launches baseline; `aether status <campaign-id>` displays state; `aether cancel <campaign-id>` stops; `aether verify <claim-id>` requests approved replay; `aether export <campaign-id>` produces manifest and report. Commands are proposed interfaces, not implemented commands.

## R2 web workspace
| Screen | Primary interaction | Failure / empty state |
|---|---|---|
| Mission setup | Select task, baseline, budget and metric | Validation errors; no execution before approval. |
| Campaign monitor | Timeline, active run, remaining caps, cancel | Reconnect with cursor; show stale status. |
| Experiment explorer | Compare attempts, metrics, failures | Show no comparable data rather than invented charts. |
| Evidence graph | Open claim, source metrics, replay, contradictions | Missing links marked INCONCLUSIVE. |
| Report review | Inspect supported and rejected claims; export | Explain blocked export and missing evidence. |
| Settings | Models, hardware, quotas and policies | No plaintext secrets in UI; confirmation for cost changes. |

## Interaction principles
Evidence before aesthetics; use simple tables and accessible charts, distinct status icons and labels (never color only), keyboard navigation, readable error messages, focus state and reduced motion. No fabricated live traces, confidence percentages or progress estimates. Human approval is a clear explicit action; show immutable protocol ID and resource caps. Browser UI must not expose hidden evaluation data.

## Practical computer applications
Software engineering: optimize a safe algorithm and verify correctness/performance. Applied ML: compare lightweight models and reproducibility. AI systems: compare model inference memory/latency under hardware caps. Academic labs: teach experimental design, provenance, negative results and peer review. Distributed systems: later compare scheduling policies under synthetic workloads. Scientific domains beyond computer science are future extensions requiring domain expertise, appropriate risk review and validated task adapters. The first release is not a general-purpose research assistant for regulated applications.

## Delivery
Native Rust CLI is the first interface. Later web UI calls Rust Axum API; Python is not a frontend or backend service. Optional Tauri desktop requires a separate threat review.
