# 21 | Native Rust implementation contract

## Workspace boundaries
| Crate | Responsibility | Internal dependencies |
|---|---|---|
| aether-core | IDs, protocol, state, evidence graph, budgets, pure ECAR ranking, metrics and verdict rules | none |
| aether-engine | Application service, ports, orchestration, recovery, report assembly | core |
| aether-store | SQLite transaction adapter, migration, blob store | core, engine |
| aether-runner | Reviewed job execution, isolation adapter, leases and bounded artifact ingestion | core, engine |
| aether-cli | Composition root, clap commands and user output | core, engine, store, runner |
| aether-api (R2) | Axum HTTP, authentication, SSE | core, engine, store, runner |

Dependencies point inward; core does not import SQLite, HTTP, Tokio or subprocess code. Engine declares ports; adapters implement them. Avoid cyclic crate dependencies. Begin with core + CLI; extract remaining crates as their milestone starts. No first-party unsafe in core.

## Worker interface and lifecycle
Controller creates an immutable job manifest in a fresh isolated workspace. Runner selects a reviewed executable and arguments from task registry, never from model text. Mount inputs read-only; no network or secrets. Worker receives job path and writes result.json and bounded artifacts in allowed output directory. Exit status is advisory; supervisor validates limits, paths, schema, attempt ID and digest before committing. Nonzero exit, missing output, invalid envelope or timeout become typed failures. Rust evaluator computes authoritative metrics on hidden data. Budget and event writes stay in Rust and are atomic. A Python worker is optional: native deterministic fixture uses same contract. JSON examples are design drafts, not executable code.

## Core invariants
- Budget values use nonnegative integer newtypes; reserve/settle checked and idempotent. No oversubscription, no unchecked casts.
- Pure ECAR score uses finite values, deterministic tie breaking, explicit replay reserve; invalid inputs fail closed. Model outputs never authorize execution or verification.
- Canonical serialization must be specified and tested across languages before digest exchange. Use raw-byte SHA-256 for artifacts; no implicit JSON field-order assumptions.
- Claim verdicts require frozen protocol, evaluator result and prescribed independent evidence. A worker cannot assert VERIFIED.
- SQLite operations transactionally bind event, attempt, usage and state. Stale lease result is rejected. Files may orphan after crash; collect only unreferenced blobs after grace period.

## R0 CLI contract (proposed)
`aether init` creates local project. `aether run-fixture` runs a fixed native experiment and stores manifest. `aether verify <run-id>` replays or verifies evidence. `aether report <campaign-id>` emits report. `aether run <protocol>` arrives only after reviewed worker integration. Exit 0 success; 2 invalid input; 3 policy denial; 4 execution failure; 5 integrity failure. JSON output option for automation. No auto-execution of arbitrary model code.

## Learning checkpoints
1. Cargo project, variables, functions, Vec and unit tests: implement accuracy from fixed labels.
2. Ownership, borrowing, structs, enums, Result: model a run with typed state transitions.
3. serde, JSON, SHA-256: freeze a manifest and detect tampering.
4. SQLite and transactions: persist evidence and atomic budget reservations.
5. Linux processes, timeouts and bounded I/O: reviewed worker; learn why Rust is not a sandbox.
6. Graph algorithms, priority queues and uncertainty: implement ECAR; compare policies.
7. Async Rust, channels and cancellation only after synchronous loop is correct; add remote compute later.

## Definition of done for first native milestone
Fresh checkout builds with locked dependencies on supported host, native fixture runs with no Python, expected metric reproduced, invalid lengths/empty input rejected, unit tests pass, README documents commands and user can explain ownership and error handling. No GPU or LLM needed. Python digits baseline follows with a reviewed worker. This package does not include application source or measured results.
