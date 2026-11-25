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

## Prometheus checks

The Windows baseline rerun at `c017c05` confirmed 33 Rust tests and three frontend tests. The later suite adds protocol compatibility and objective-planning coverage, independent maximum-subarray verification, rejected evidence, cross-process admission, current snapshot semantics, adapter contracts, bounded evolutionary search and hardware parsing.

| Command or drill | Observed outcome |
|---|---|
| `cargo fmt --all -- --check` | Exit 0 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo test --locked --workspace` | 50 passed |
| `cargo build --locked --release --workspace` | Exit 0 |
| `npm --prefix web run lint`, `run check`, `test`, and `run build` | Lint and TypeScript passed, three frontend tests passed, and Vite built production assets |
| Browser | Chromium desktop and mobile passed keyboard login, sorting and maximum subarray campaigns, verdicts, graph, planner, metadata gates, benchmarks, candidate archive, report and driver panel |
| Release exports | The sorting, maximum-subarray and earlier objective reports matched 84,449, 81,655 and 85,870 original bytes. A fresh version 3 campaign reused the first advice, rejected execution before approval and exported a byte-identical report. |
| Live model gate | The installed Ollama service answered preflight. The release CLI rejected its cloud-enabled configuration before any planning request. |
| Known-advisory checks | See [current dependency audit](../governance/DEPENDENCY_INVENTORY.md). |

An interrupted maximum-subarray release probe observed a live persisted attempt lease, terminated its own temporary supervisor, and tried recovery. The early request was rejected. After the lease expired, recovery marked one attempt expired, charged the reservation and preserved the journal without a verified result. The CLI recovery call took 173.39 ms after eligibility.

The first NVIDIA environment-isolation attempt failed to initialize NVML. A direct probe isolated the required `ProgramFiles` addition. The next release reported the physical RTX 3050 with only `SystemRoot` and `ProgramFiles` retained. [GPU profile](../operations/PROMETHEUS_GPU_PROFILE.md)

The default browser service denied unauthenticated hardware access, path-shaped assets and outbound metadata requests. A separately opted-in release service exercised each provider with public bibliographic queries and retrieved the temporary catalog entries. [Literature validation](../research/LITERATURE_ADAPTERS.md)

## Athena Windows checks

The final x64 MSVC release executable SHA-256 is `a6819ae85b420ca3bf6af582fa5ee9e7367b90e57aeea3aa4bbae76ae3fee61c`.

| Check | Observed result |
|---|---|
| `cargo fmt --all -- --check` | Exit 0 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo test --locked --workspace` | 53 passed. Two ignored Windows child fixtures ran inside the descendant containment test. |
| `cargo build --locked --release --workspace` | Exit 0 |
| `npm --prefix web run lint`, `run check`, `test`, `run build` | Lint and type checks passed, all three tests passed, and production assets built. |
| `node tests/browser-smoke.cjs` | Desktop and mobile Chromium passed sorting and optimization, evidence, offline planner, reports, metadata gates, benchmark and GPU panel. |
| Cited draft browser check | The real local draft rendered in desktop and 390-pixel mobile Chromium. The protocol API denied unauthenticated access. |
| Release report compatibility | Four earlier exports and the frozen Athena draft matched byte for byte, after committing the recorded draft with the release CLI's LF line endings. |
| Scientific replay | Two fresh processes matched all 1,440 development outcome rows and the analysis reproduced its saved values. The held-out specification remains unexecuted. |
| Dependency advisories | RustSec checked 202 lock entries and npm checked 82. Both exited 0 with zero known findings. [Exact lock and advisory database](../governance/DEPENDENCY_INVENTORY.md#athena-lock-review) |
| Job Object cleanup | A running child fixture started its descendant, which reached a marker. Closing the job prevented the second marker and the test reaped the parent. |
| Recovery drill | Terminated a fresh temporary supervisor after observing its persistent active lease. Early recovery refused. After expiry, recovery settled one attempt with 2,000 ms charged, no verified claim, and a failed campaign. The eligible recovery invocation took 71.23 ms on the initial Athena Windows release. |
| `cargo test --locked -p aether-api` | 22 passed across the API unit and integration suites, including citation binding, local model provenance, rejection before approval and fresh replay. |
| Approved model-guided run | The exact pinned release ran three successful attempts. Original and fresh replay had matching input hashes. The seeded challenge had a distinct input hash. All three exact-reference metrics were 1.0, and the pinned trusted verifier marked the claim verified. The run used 140/12,000 ms and 3/5 attempts. The report re-export matched saved bytes. [Report](../../research/results/athena-local-campaign.json) |

The final provider regression check confirms that a missing manifest digest rejects the local provider before generation. The first parallel attempt at this test stalled while the two loopback fixtures competed for the one provider slot. A test-only lock now serializes their startup, and both passed in the final suite.

The release compatibility check read the original campaign state. The browser smoke and recovery drill used separate fresh fixtures. The campaign execution used the owner approved original binary and did not modify its frozen protocol. [Frozen draft](../../research/results/athena-local-draft-report.json) · [Worker containment](../security/ATHENA_WORKER_CONTAINMENT.md)
