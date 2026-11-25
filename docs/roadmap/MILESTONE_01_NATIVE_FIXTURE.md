# Milestone 01 — native classification fixture

Status: authorized under implementation-first development. Scope is Rust 2024, core evaluator and CLI with minimal local manifest storage. The fixture uses synthetic data and runs locally without Python, GPU or model calls. Validate the code and teach the implementation through the standalone practice workspace and code tour.

## Frozen exercise proposed for implementation

Fixture `classification-v1` uses ground truth `[0, 1, 1, 0]` and predictions `[0, 1, 0, 0]`. Accept nonempty slices of equal length and labels from {0, 1}. Count matches using one pass. Return integer `correct` and `total`. Display their quotient as accuracy (3/4 = 0.75 for this fixture). Exact count comparison governs replay. Labels and predictions are known teaching inputs. A fresh process replay checks determinism and integrity, and cannot establish an independent scientific verdict.

## Interfaces

| Command | Input and preconditions | Output and state |
|---|---|---|
| `aether init` | Writable current directory, no foreign state at local project path | Initialize versioned local project metadata and display its ID. Repeating init returns the same ID without rewriting valid state. Reject a conflicting or malformed project directory. |
| `aether run-fixture` | Initialized local project, built-in fixture | Create an immutable attempt under a local fixture campaign. Record run/campaign IDs, fixture/evaluator version, input and manifest digests, counts and status. Report IDs and 3/4. |
| `aether verify <run-id>` | Existing well-formed run ID and compatible manifest | Validate digest and versions, recompute in a fresh CLI invocation and report replay pass or failure. Preserve prior attempt bytes. |
| `aether report <campaign-id>` | Existing well-formed campaign ID | Render counts and attempt outcomes with manifest references in stable order. Validate records and fail if integrity checks fail. |

Source exit codes apply: 0 success, 2 invalid input including missing ID, 3 policy denial, 4 I/O or execution failure, 5 corrupted or incompatible evidence. Keep errors concise on stderr. Define successful output and errors before introducing a machine-readable output flag. Never build storage paths from raw unconstrained IDs. Use local per-project IDs and one documented commit point per attempt. An interrupted write cannot appear as a completed attempt. Phase 2 introduces the full journal and claims.

## Acceptance

| ID | Evidence expected |
|---|---|
| M1-01 | Fresh local checkout builds with a documented Rust toolchain and lockfile. Run the native fixture in an environment without Python, a model API or GPU access. |
| M1-02 | Unit tests cover exact 3/4, all match, no match, empty input, mismatched lengths and an invalid label. Errors are returned with Result. |
| M1-03 | Init is repeatable. Run, verify and report work across fresh CLI invocations. Two runs produce distinct attempt IDs and the same counts. |
| M1-04 | A changed manifest byte, bad ID or interrupted write produces a typed failure or exposes no completed attempt. Earlier valid attempts remain readable. |
| M1-05 | Command help and usage examples cover preconditions and exit statuses. The learning walkthrough explains an input borrow, count versus quotient, invalid input and the verification boundary. |

Run checks are specified in the [test plan](../testing/TEST_PLAN.md). Begin with [Lesson 1](../learning/LESSON_01_CLASSIFICATION_AND_RUST.md).

The application workspace must eventually pass the following gates. Temporary probe outcomes do not count toward these gates.

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```
