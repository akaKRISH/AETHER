# Component and crate architecture

Arrows in the [dependency diagram](../diagrams/crate-dependencies.mmd) point from importer to dependency.

| Crate | Responsibility | Internal dependencies |
|---|---|---|
| aether-core | IDs, protocol, state, budget, evidence, pure ranking and trusted metrics | None |
| aether-engine | Workflow, ports, durable transitions, recovery and report assembly | Core |
| aether-store | SQLite, migrations, artifact storage, transactional events | Core, engine |
| aether-runner | Reviewed tasks, bounded intake and isolation adapter | Core, engine |
| aether-cli | Native command and composition boundary | Core, engine, store, runner as activated |
| aether-api | R2 HTTP, authentication and SSE | Core, engine, store, runner |

Start with core and CLI. Give fixture manifest I/O a small CLI-local boundary and relocate it when durable storage has a use case. Verification rules live in core, while trusted input loading and replay are engine duties. Compute and scientific memory remain later interfaces. Python enters exclusively through approved worker contracts.

Verify layering with Cargo metadata once manifests exist and by building core without runtime or storage dependencies. See the [source native design](../archive/v0.2/AETHER/21_RUST_ENGINEERING.md).
