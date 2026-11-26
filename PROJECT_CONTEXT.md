# AETHER project context

AETHER is the Autonomous Research and Engineering Laboratory. It aims to produce reproducible computational evidence with a bounded, human-governed workflow. The owner is a B.Tech CSE student learning systems and research engineering from first principles. Goals include a defensible portfolio, publishable research if supported by evidence, an open-source release if approved, and eventual customer discovery. Planning cadence depends on learning progress and available time.

## Source of truth

The [Rust v0.2 design package](docs/archive/v0.2/AETHER/README.md) and [Project Genesis brief](docs/archive/genesis/PROJECT_GENESIS.md) govern Phase 0. The [decision log](docs/governance/DECISION_LOG.md) records acceptance status. [Source reconciliation](docs/SOURCE_RECONCILIATION.md) records how Genesis reads ambiguous source passages. The [master blueprint](docs/AETHER_MASTER_BLUEPRINT.md) links the live design. [Project status](PROJECT_STATUS.md) tracks execution and outstanding owner decisions.

## Technical model

- The native CLI calls the deterministic Rust core for type validation and metrics. The engine holds pure policy and scheduling. The store and runner implement persistence and execution. The local API composes the controller and serves the React workbench.
- A protocol pins the task, evaluator, data, limits and approval. A campaign follows that protocol. Attempts produce bounded outputs and immutable manifests. Content digests identify bytes. An evidence graph links claims to metrics, artifacts, replays and dependencies.
- A SQLite transaction binds accepted events, state and budget settlement. The verified Windows build supervises trusted built-in workloads. A supported profile for stronger OS isolation remains open.
- Athena qualifies local Qwen on a dedicated cloud-disabled Ollama process. A version 4 protocol freezes advisory output and reviewed citation snapshots. The owner approves the exact digest before any task runs. [Inference record](docs/operations/ATHENA_INFERENCE.md) · [ADR-013](docs/decisions/ADR-013.md)
- ECAR selects EXPLORE, REPLAY, CHALLENGE or STOP. The scheduler ranks eligible actions, while the verifier rules on claims. Scientific comparison holds resource caps, task information and final evaluation constant. See the [ECAR specification](docs/research/ECAR_SPECIFICATION.md).
- APOLLO keeps literature metadata revisions, citation links and verbatim source passages in the local catalog. Extracted passages are unverified and have no execution or approval authority. [ADR-014](docs/decisions/ADR-014.md)
- The scientific knowledge graph preserves source and version references for typed entities and relationships. A captured verification receipt stays scoped to its authoritative campaign record and capture time. [ADR-015](docs/decisions/ADR-015.md)
- Hypothesis discovery ranks unverified proposals over bounded graph slices. Each proposal records its test design, limits and exact supporting sources. [ADR-016](docs/decisions/ADR-016.md)
- Experimental design turns three archived hypotheses into canonical, unapproved exploratory drafts. Campaign freeze, approval and independent verification keep their existing authority. [ADR-017](docs/decisions/ADR-017.md)
- The model laboratory indexes local model headers, verifies file identities and checks public development responses under separate inference and evaluation budgets. [ADR-018](docs/decisions/ADR-018.md)
- Adapter-training manifests pin checked dataset splits and base identity. The current controller only rehearses synthetic checkpoint receipts. [ADR-019](docs/decisions/ADR-019.md)
- The workspace host has Windows 11, i5-13420H (8 cores, 12 logical processors), 15.64 GiB observed system RAM and an RTX 3050 Laptop GPU with 6144 MiB reported VRAM. Ubuntu WSL2 is registered. Verify tooling and isolation before any external worker. [Environment snapshot](docs/operations/DEVELOPMENT_ENVIRONMENT.md)

## Sequence and boundaries

The native classification fixture and fresh-process replay are the foundation. The local workbench adds reviewed sorting and maximum-subarray campaigns, evidence storage, ECAR, and optional model proposals. Prometheus brings individually configured model and literature adapters into the local architecture. Model suggestions have no worker, credential or verdict permissions. Remote compute, hosted access, an external worker SDK and held-out studies have additional gates. Use only reviewed, lawful, benign computational tasks. Keep protected information outside tracked data and outputs.

DAE-01 adds standalone local discovery using the reviewed knapsack fixture. Its lineage report uses the existing evidence store. Campaign promotion remains governed by the frozen protocol. [Discovery checkpoint](RESUME.md#dae-01-constrained-discovery)

Student exercises run in the standalone [Rust practice workspace](learning/README.md). Production evaluation belongs in approved crates and is validated against the milestone criteria.

The owner requires concise, maintainable code across the stack. Prefer a compact expression when it preserves clarity. Keep safety and scientific invariants explicit, preserve stable behavior during related refactors, and measure performance claims. The approved priority is correctness, security, clarity, performance, then conciseness. [Coding standards](docs/engineering/CODING_STANDARDS.md)

The owner authorized implementation-first development across the eligible milestones. Build and validate each coherent slice, then teach and document it through the actual code. Student exercises remain available in parallel. Update this context when architecture, assumptions or phase prerequisites change. Use [project status](PROJECT_STATUS.md) for current evidence and [resume](RESUME.md) for the active checkpoint.
