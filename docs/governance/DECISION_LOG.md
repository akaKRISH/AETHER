# Decision log

| Item | Recorded decision | Change control |
| --- | --- | --- |
| Authority order | Rust v0.2 controls technical design. The Genesis brief defines scope, deliverables and approval. v0.1 is historical. | Record conflicts and explain resolutions before implementation. |
| Rust first | Owner-authorized design direction from v0.2. | Superseding ADR with measured design evidence and owner approval. |
| Native milestone first | Owner brief requires fixed deterministic Rust classification before optional Python-backed R0 task pack. | A Python integration has its own worker, licensing and isolation gate. |
| Phase 1 authorization, 24 November 2026 | Owner provisionally accepted Genesis and authorized Milestone 01 preparation. The design check found the controller boundary consistent with ADR-009. The proposed starting members are core and CLI. | Review the first student attempt and obtain explicit approval before adding application source or Cargo members. |
| Phase 1 activation, 25 November 2026 | Owner accepted the SDK validation, completed Phase 0 and authorized Phase 1 under the learning-first policy. Exercises use a dedicated Cargo workspace. | Review each student submission before advancing. Follow the existing design and approval checkpoint for production components. |
| Implementation-first authorization | Owner superseded the lesson and assessment pauses and approved successive local engineering milestones. Keep the separate practice workspace and derive lessons from tested code. | Preserve separate approval gates for destructive operations, spending, public deployment, security-sensitive changes and major architecture changes. |
| Local first | The repository, practice workspace and build checks are local. | Review and approve each publication, deployment or spending action separately. |
| Bounded workspace storage | The local workbench uses content-addressed SQLite BLOBs within campaign transactions. The native CLI keeps its separate immutable file layout. | [ADR-011](../decisions/ADR-011.md) records the trade-off, restore evidence and trigger for measuring a sidecar store. |
| Licence | Apache-2.0 is proposed. Code, documentation and third-party assets require scoped review. | [License decision](../../LICENSE_DECISION.md) and ADR-008. |
| ECAR study | Proposed research program with prior-art, pilot, frozen main protocol and independent verification gates. | Maintain preregistration and dated deviation records. |
| Mentor and reviewer | Faculty mentor and independent results reviewer remain unassigned. | Owner identifies and records role scope before dependent gates. |

For a future entry record date, owner, rationale, alternatives, evidence, affected requirements, and whether it needs supersession or expiration. [Decision register](ARCHITECTURE_DECISION_RECORDS.md)
