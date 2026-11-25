# AETHER

AETHER is the Autonomous Research and Engineering Laboratory. It is designed to help researchers run bounded computational experiments, trace every quantitative claim to evidence, and test whether allocating resources to verification improves results.

**Status:** Implementation-first. The local Rust platform includes a native fixture CLI, evidence storage, ECAR and a loopback workbench. See [project status](PROJECT_STATUS.md) for verified scope and remaining work.

## Run locally

```powershell
cargo run --locked --release --bin aether-lab -- demo sorting-demo 19
```

This runs the reviewed sorting candidate, a fresh-process replay and a distinct frozen challenge. It stores the campaign, budget, decision traces and verdict in the local data directory. Use the [local launch guide](docs/operations/LOCAL_WORKBENCH.md) for separate planning and approval, native fixture commands, the browser workbench and recovery.

- Start with the [master blueprint](docs/AETHER_MASTER_BLUEPRINT.md) and [project status](PROJECT_STATUS.md).
- Read the [controlling Rust v0.2 source](docs/archive/v0.2/AETHER/README.md) and [source inventory](docs/archive/README.md).
- Work through [Lesson 1](docs/learning/LESSON_01_CLASSIFICATION_AND_RUST.md) in the [Rust practice workspace](learning/README.md). Review the [native milestone contract](docs/roadmap/MILESTONE_01_NATIVE_FIXTURE.md).
- Learn the actual implementation through the [codebase tour](CODEBASE_TOUR.md) and [rebuild curriculum](docs/learning/REBUILD_AETHER_FROM_SCRATCH.md).
- See [how to contribute](CONTRIBUTING.md), the [security contact status](SECURITY.md), and the [pending license decision](LICENSE_DECISION.md).

Rust Edition 2024 owns authoritative decisions and state. The first fixture runs with a local CPU and fixed data. Later, reviewed Python workloads can submit evidence through versioned contracts. The trusted Rust evaluator decides whether that evidence meets the protocol.
