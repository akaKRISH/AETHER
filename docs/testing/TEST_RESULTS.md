# Implementation validation

The verified host is the Windows x64 MSVC laptop with Rust and Cargo 1.96.1. See the [environment record](../operations/DEVELOPMENT_ENVIRONMENT.md).

| Area | Evidence |
|---|---|
| Native evaluator | Exact 3/4, all-match and zero-match counts, empty and unequal inputs, invalid binary labels |
| Native CLI | Idempotent and concurrent initialization, coexistence with workbench state, separate-process replay, immutable published records, tampered inputs fail |
| Evidence persistence | Reopen, atomic rollback, stored digest check, verification and dependency revocation, lease fencing, budget overrun and residual reserve |
| ECAR policy | Exact ranking, stable ties, checked arithmetic, reproducible A–E simulations |
| Worker | Approved seeded sorting variants against a separate reference, rejected malformed or mismatching results, cancellation before launch |
| Loopback HTTP | Denied unauthenticated reads and streams, resumed SSE from a committed sequence, refused unapproved start and default-off external queries, completed an authenticated campaign, read verified artifact bytes, and handled repeated cancellation of a draft |
| Graceful shutdown | A separate Windows console delivered Ctrl-C to the release service with an authenticated SSE connection open and idle. The stream returned EOF and the process exited 0 in 0.458 s. |
| Active-worker cancellation | A Windows release drill observed the approved sorting child process, requested cancellation through the API and confirmed the supervisor reaped that child. The attempt settled as cancelled, the budget charged the attempt and the campaign produced no verified claim. |
| Crash recovery | Terminated a temporary release supervisor while its reviewed worker was alive and observed the child exit. Early recovery refused the unexpired lease. After expiration it closed the campaign as failed, expired and charged one attempt, appended its journal entry and produced no verified claim. |
| Backup and restore | Restored the recorded campaign from a temporary SQLite backup and frozen protocol copies. The release CLI exported the same 84,449 bytes before and after restore with SHA-256 `55ef0a797a07c7ce2e01538c3dd67e7e030621e14cd8b2bb19949cd8e1159b08`. |
| Browser | Chromium at 1366×900 and 390×844. Connected with a token, created and approved a draft, ran the campaign, inspected evidence, replay and ECAR decisions, loaded an export, generated offline proposals, found CAVEAT in the catalog, and rendered the synthetic benchmark. No uncaught browser errors or mobile horizontal overflow in this run. |

RustSec and npm dependency review results are in the [dependency inventory](../governance/DEPENDENCY_INVENTORY.md). Security tests cover the reviewed local execution profile. [Security review](../security/SECURITY_REVIEW.md)

## Final run

| Command | Observed result |
|---|---|
| `cargo fmt --all -- --check` | Exit 0 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo test --locked --workspace` | 33 Rust tests passed |
| `cargo build --locked --release --workspace` | Exit 0 |
| `npm --prefix web test` | 3 frontend tests passed |
| `npm --prefix web run check` | TypeScript check passed |
| `npm --prefix web run build` | Production assets built |
| `node tests/browser-smoke.cjs` | Local Chromium smoke passed |
| `scripts/start-local.ps1 -Port <free-port>` | In a temporary Windows session, installed locked web dependencies, built the interface, and served health, the index and authenticated runtime info |

The release `aether` executable ran `init`, `run-fixture`, `verify` and `report` using the default repository with the workbench present. Both evaluation and replay reported 3/4. An earlier smoke exposed the shared state directory conflict. The corrected initializer accepts the workbench's sibling directory and has a regression test.

These runs cover Windows x64 only. Live Ollama inference, live Crossref retrieval, Linux, frontend lint tooling, the deferred Python worker, and OS-level isolation qualification remain unverified. See the [continuation record](../../NEXT_ACTIONS.md) and [recorded development experiments](../research/BENCHMARK_RESULTS.md).
