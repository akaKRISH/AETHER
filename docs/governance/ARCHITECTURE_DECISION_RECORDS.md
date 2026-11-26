# Architecture decision records

An architecture decision record documents the decision, alternatives considered, trade-offs, status and conditions for revisiting it. ADR-009 records the Rust-first constraint authorized by the owner and reiterated at Genesis. Other v0.2 decisions are proposed design directions pending milestone review. A proposal may guide documentation and still require approval before implementation.

| Record | Direction | Status |
| --- | --- | --- |
| [ADR-001](../decisions/ADR-001.md) | CLI-first modular monolith | Proposed |
| [ADR-002](../decisions/ADR-002.md) | SQLite and content-addressed artifacts | Proposed |
| [ADR-003](../decisions/ADR-003.md) | Inspectable ECAR scheduler | Proposed |
| [ADR-004](../decisions/ADR-004.md) | Trusted external verification | Proposed realization of an authoritative v0.2 invariant |
| [ADR-005](../decisions/ADR-005.md) | CPU-first native fixture | Proposed footprint for an authoritative v0.2 constraint |
| [ADR-006](../decisions/ADR-006.md) | Reviewed tasks and bounded autonomy | Proposed realization of authoritative scope |
| [ADR-007](../decisions/ADR-007.md) | Frozen protocols and append-only history | Proposed realization of authoritative v0.2 invariants |
| [ADR-008](../decisions/ADR-008.md) | Apache-2.0 candidate | Proposed, unadopted |
| [ADR-009](../decisions/ADR-009.md) | Rust-first trusted control plane | Accepted design authority |
| [ADR-010](../decisions/ADR-010.md) | Versioned JSON worker boundary | Proposed |
| [ADR-011](../decisions/ADR-011.md) | Transactional SQLite BLOBs for bounded workbench artifacts | Implemented local scope |
| [ADR-012](../decisions/ADR-012.md) | Reviewed optimization task and optional provider adapters | Authorized local engineering scope |
| [ADR-013](../decisions/ADR-013.md) | Cited objective drafts with qualified local model provenance | Athena implementation |
| [ADR-014](../decisions/ADR-014.md) | Versioned literature archive and exact source passages | APOLLO local scope |
| [ADR-015](../decisions/ADR-015.md) | Typed graph in the SQLite literature catalog | APOLLO local scope |
| [ADR-016](../decisions/ADR-016.md) | Deterministic proposals from bounded graph slices | APOLLO local scope |
| [ADR-017](../decisions/ADR-017.md) | Canonical experimental design drafts and campaign authority | APOLLO local scope |
| [ADR-018](../decisions/ADR-018.md) | Local model metadata, evaluation and admission | APOLLO local scope |

Preserve historic reasoning. Record changes with a superseding ADR and update relevant requirements, contract versions and diagrams in the same review. Document areas where the executable acceptance criterion still needs refinement. [v0.2 ADR and traceability](../archive/v0.2/AETHER/19_ADR_TRACEABILITY.md)
