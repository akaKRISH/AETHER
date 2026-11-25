# Project status

25 November 2026 · **Implementation-first local workbench.** The owner approved successive implementation milestones and superseded the lesson pauses. The existing Rust-first architecture and security approval boundaries govern the work.

## Completed

- Reviewed both handoff guides, all 21 v0.2 chapters, the 24-page PDF, 34 ZIP entries, original contract drafts and diagrams.
- Archived literal source bytes and preserved the historical Python-first files.
- Created the project policies, designs, research plan, learning materials and Phase 1 contract. Initialized local Git on main.
- Performed the documentation and host checks recorded in the [documentation audit](docs/DOCUMENTATION_AUDIT.md).
- Confirmed Milestone 01 aligns with Rust v0.2. The proposed initial workspace has a pure Rust 2024 core and a thin CLI. The fixture's separate-process replay checks determinism and manifest integrity.
- Diagnosed the native Windows build with `vswhere` and a dependency-free Rust 2024 probe. Cargo check, Clippy and formatting passed. Debug build, release build and test linking failed with `LNK1181` for `kernel32.lib`. No release executable ran. [Probe results](docs/operations/DEVELOPMENT_ENVIRONMENT.md#windows-sdk-repair-diagnosis)
- Installed the authorized Microsoft Windows 11 SDK 26100-series desktop x64 feature. The replacement probe passed all six Cargo checks and ran the release executable with exact output in the x64 developer environment. Fresh PowerShell 7 and Windows PowerShell 5.1 also built and ran the executable. Native Windows toolchain qualification is unblocked. [Repair verification](docs/operations/DEVELOPMENT_ENVIRONMENT.md#windows-sdk-repair-and-verification)
- Set up a standalone Rust 2024 [practice workspace](learning/README.md). The dependency-free Lesson 1 starter passed a locked offline build and format check.
- Built the native fixture CLI and pure classification evaluator, SQLite evidence and budget store, exact ECAR scheduler, reviewed worker, authenticated loopback API and React workbench. Added local model and citation interfaces, synthetic comparison and native timing commands, and a [codebase tour](CODEBASE_TOUR.md).
- The Windows native workspace passed formatting, strict Clippy, 33 tests and the release build. The native fixture reported 3/4 and independently replayed it. The release sorting campaign completed with three attempts, one replay pass, one verified claim and one proposed follow-up claim. A temporary restore produced the byte-identical release report. [Test results](docs/testing/TEST_RESULTS.md) · [Benchmarks](docs/research/BENCHMARK_RESULTS.md)
- The web interface passed three frontend tests, type checking, production build and an authenticated browser campaign on desktop and mobile. The API integration also exercised event-stream resume and repeated draft cancellation. A Windows Ctrl-C drill closed an open idle stream and exited cleanly. [Test results](docs/testing/TEST_RESULTS.md)

## Decisions and next actions

ADR-009 (Rust Edition 2024 trusted architecture) remains the accepted direction. Exercises remain available in parallel. Start with the tested code in the [rebuild curriculum](docs/learning/REBUILD_AETHER_FROM_SCRATCH.md) or the existing [Lesson 1](docs/learning/LESSON_01_CLASSIFICATION_AND_RUST.md).

The local workbench runs reviewed seeded sorting and deterministic fixture tasks. OS-enforced hostile worker isolation, external datasets, hosted authorization, distributed workers, GPU workload accounting and independent held-out evaluation remain open. Live Ollama generation and Crossref retrieval were not verified. [Security review](docs/security/SECURITY_REVIEW.md) · [Continuation](NEXT_ACTIONS.md)

The recorded experiments are local development checks on synthetic inputs. ECAR efficacy and the comparison with close prior work including CAVEAT remain open research questions. [Originality assessment](docs/research/ORIGINALITY_ASSESSMENT.md)

[Blueprint](docs/AETHER_MASTER_BLUEPRINT.md) · [Decisions](docs/governance/DECISION_LOG.md) · [Reconciliations](docs/SOURCE_RECONCILIATION.md)
