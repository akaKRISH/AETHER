# Rebuild AETHER from scratch

The goal is to reconstruct the ideas in a new practice program and explain every decision. Use the separate [learning workspace](../../learning/README.md) for exercises. Keep research data and durable state from the real application outside your experiments. Work through the [codebase tour](../../CODEBASE_TOUR.md) once before rebuilding.

## Establish a feedback loop

1. Predict the result by hand. Write the smallest failing test that can distinguish your intended behavior from a plausible mistake.
2. Implement the smallest working version. Run the local test and explain the mechanism aloud before comparing the corresponding AETHER function.
3. Intentionally break one invariant in your practice implementation. Reproduce the failure with the test, isolate the first incorrect value, restore the invariant, and run the test again.
4. Write a short note describing your design, the failure you can detect, and the next alternative you would test. Compare measured observations under the same input and build profile.

## Build in this order

| Stage | Rebuild unit | Definition of done | Source-backed lesson |
| --- | --- | --- | --- |
| 1. Rust foundations | CLI that prints four pairs and counts matching binary labels using borrowed slices | Hand example, unequal lengths, empty input and invalid labels pass tests | [Rust and metrics](IMPLEMENTATION_RUST_EVALUATOR.md) |
| 2. Replay | Serialize a toy manifest and read bytes through a stored digest reference | One-byte mutation fails integrity and a clean rerun repeats exact counts | [Rust and metrics](IMPLEMENTATION_RUST_EVALUATOR.md) |
| 3. Durable state | SQLite campaign, attempt and budget tables with one reserve/settle transaction | Repeated same settlement is a no-op and inconsistent settlement fails | [Storage and graphs](IMPLEMENTATION_STORAGE_GRAPHS.md) |
| 4. Claim graph | An acyclic edge list and walk that invalidates downstream claims | Remove evidence from A in A → B → C and explain what changes in the projection and what remains in history | [Storage and graphs](IMPLEMENTATION_STORAGE_GRAPHS.md) |
| 5. Scheduling | Rank two typed action costs using exact ratios and one stable tie-break | Reorder candidates, hit a cap and keep a replay reserve | [ECAR and statistics](IMPLEMENTATION_ECAR_STATISTICS.md) |
| 6. Process supervision | Run a known program in a bounded child process | Validate identity of a small result, exercise timeout with a sleeping fixture and reap the child | [Operating systems and security](IMPLEMENTATION_OS_SECURITY.md) |
| 7. API and UI | Protected read endpoint plus one browser table and a draft-only planner | Valid token reads snapshot, absent token gets 401, and typed planner output cannot launch anything | [API and agents](IMPLEMENTATION_API_AGENTS.md) |
| 8. Study and compute | Run a matched synthetic comparison, then record one separately labelled end-to-end timing exercise | Preserve all paired attempts and prove which quantities are modeled and which you measured | [ECAR and statistics](IMPLEMENTATION_ECAR_STATISTICS.md) and [Compute](IMPLEMENTATION_COMPUTE_DISTRIBUTED.md) |

## First practice run

Start at [Lesson 1](LESSON_01_CLASSIFICATION_AND_RUST.md). Open its starter, predict the requested result and run from the repository root.

```powershell
cargo run --manifest-path .\learning\Cargo.toml --bin lesson_01
cargo test --locked -p aether-core
```

The first command runs your practice program. The second checks the repository evaluator. For an end-to-end walkthrough on a fresh local state directory, choose a new path and set `AETHER_DATA_DIR` in the current PowerShell process. Then compare the controlled demo and synthetic benchmark. This example creates local data under a new temporary directory.

```powershell
$env:AETHER_DATA_DIR = Join-Path ([IO.Path]::GetTempPath()) ('aether-study-' + [Guid]::NewGuid().ToString('N'))
cargo run --locked --bin aether-lab -- demo practice-sort 19
cargo run --locked --bin aether-lab -- report practice-sort
cargo run --locked --bin aether-lab -- benchmark
```

Locate the worker receipt digest and protocol digest in the report. Describe why each can match on repeated reads while the verdict still depends on a frozen evaluator and replay. Use a fresh shell for unrelated AETHER work.

## Capstone inspection

Draw your trust boundaries from memory. Identify which thread or process has authority to write the ledger, name the reservation and completion transactions, and explain the three ways a result can remain unusable as evidence. Demonstrate a benign malformed-input or stale-lease test in your practice project. Then propose one change that preserves the existing dependency direction and one experiment that could falsify your own performance hypothesis. Course progress and implementation milestones proceed independently.
