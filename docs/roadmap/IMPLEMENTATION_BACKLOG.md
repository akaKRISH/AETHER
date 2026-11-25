# Implementation backlog

The owner authorized implementation-first development. Use [the continuation record](../../NEXT_ACTIONS.md) for the current implementation order and pending approval gates. The original first-slice planning table below records how the native fixture was scoped.

| Order | Ready when | Slice | Acceptance reference |
|---|---|---|---|
| 1 | Owner approves M1 | Establish the smallest Cargo workspace with core and CLI shells; lock toolchain/dependencies after compatibility check | M1-01 |
| 2 | Owner works through fixture | Implement pure metric, typed errors, borrowed inputs and unit check | M1-02 |
| 3 | Metric reviewed | Add init and run with safe manifest commit | M1-03, M1-04 |
| 4 | Manifest format agreed | Add integrity check, deterministic replay and evidence report | M1-03, M1-04, M1-05 |
| 5 | M1 accepted | Detail persistence/event schemas, SQLite migration plan and graph policy | Phases 2 onward in roadmap |

No feature work is active during Phase 0. Use [milestone acceptance](MILESTONE_01_NATIVE_FIXTURE.md) and [traceability](../testing/REQUIREMENTS_TRACEABILITY.md) when turning rows into work items.
