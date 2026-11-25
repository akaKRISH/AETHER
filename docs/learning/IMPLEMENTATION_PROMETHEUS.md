# Rebuild the Prometheus components

Use the separate [practice workspace](../../learning/README.md). Each exercise has a visible outcome and can be attempted independently of development.

| Idea | Source to follow | Small rebuild |
|---|---|---|
| Maximum subarray | `solve` and `compare` in the [reviewed worker](../../crates/aether-runner/src/optimization.rs) | Enumerate intervals of `[-2, 4, -1, 3]` and write a second implementation that remembers the best interval ending at each position. Compare scores and intervals on all-negative inputs. |
| Frozen challenge | `PlanRequest::optimization_inputs` and `Lab::run_approved` in the [controller](../../crates/aether-api/src/lab.rs) | Store two arrays under an exact hash. Run the original twice and the challenge once. Explain why three runs represent only two distinct inputs. |
| Evolutionary search | `search` and `exhaustive_optimum` in the [engine](../../crates/aether-engine/src/evolution.rs) | Start with four items and use a bit mask for the selected subset. Flip one bit to make a child, retain its parent ID and compare its value and capacity. Exhaustively verify the best value. |
| Model adapter | `propose_recorded` in the [planner](../../crates/aether-api/src/planning.rs) | Write a local fake provider returning a valid sort name, a duplicate, and an unexpected execution field. Test rejection of both invalid responses before converting to a draft. |
| Metadata normalization | `search_metadata` in [discovery](../../crates/aether-api/src/literature/discovery.rs) and `CitationCatalog` in the [catalog](../../crates/aether-api/src/literature.rs) | Normalize the two spellings of a DOI and check that they map to one entry. Keep a preprint identifier and its separately published journal identifier distinct. |
| Snapshot isolation | `Store::initialize` and `Store::snapshot` in the [storage crate](../../crates/aether-store/src/read.rs) | Hold a write open in one connection. Read the last committed claim and audit event from a second connection. Commit and check the new pair. |
| Performance and hardware | [Criterion scheduler harness](../../crates/aether-engine/benches/scheduler.rs), [loopback timer](../../tests/measure-loopback.cjs), and `gpu_snapshot` in [hardware.rs](../../crates/aether-api/src/hardware.rs) | Measure one pure function and one HTTP read. Compare warmup, units, and what each timer includes. Report an absent GPU reading explicitly. |

Interview practice. Explain why an independently computed score is useful even when both methods share the same binary. Identify where time and token usage are recorded. Describe a disagreement the program could observe and what additional evidence would justify a stronger claim.
