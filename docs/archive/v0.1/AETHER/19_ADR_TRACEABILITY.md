# 19 | Architecture decisions and requirements traceability

| ADR | Decision / rationale | Revisit when |
|---|---|---|
| ADR-001 | CLI-first modular monolith; fewer failure modes and teachable code. | Multiple users or proven bottleneck. |
| ADR-002 | SQLite + content-addressed local blobs; simple audit and replay. | Write concurrency exceeds local profile. |
| ADR-003 | Custom ECAR scheduler; core research variable must be inspectable. | Framework offers equivalent transparent controls. |
| ADR-004 | Deterministic external verifier; no agent self-certification. | Never relax without scientific protocol change. |
| ADR-005 | CPU-first, small datasets; affordable reproducibility. | GPU benefit demonstrated. |
| ADR-006 | Restricted reviewed tasks; no unrestricted self-modification. | New task requires explicit safety review, not automatic relaxation. |
| ADR-007 | Immutable protocol hashes and append-only audit; traceable claims. | Storage redesign preserves semantics. |
| ADR-008 | Proposed Apache-2.0; contributor-friendly. | Owner/license review before release. |

## Traceability matrix
| Requirement | Module / design | Verification |
|---|---|---|
| PRD-F01,F02 | protocol, controller | T01,T02 |
| PRD-F03,F04 | runner, ledger, evaluator | T03,T05,T06 |
| PRD-F05 | runner, budget | T08,T09 |
| PRD-F06,F07 | scheduler, budget | T01,T04,T15 |
| PRD-F08,F09 | evaluator, report | T05,T07,T15,T16 |
| PRD-F10,F11,F12 | API/UI/adapter | T13,T14; future remote/SDK tests |
| TRD-N01,N02,N03,N04 | CPU runner, budget, artifacts | T03,T04,T09,T05 |
| TRD-N05,N06,N07 | API, isolation, leases | R2 load test,T10,T08 |
| TRD-N08,N09,N10 | backup, events, UI | T12,T03/T08,T14 |
| SEC-01 through SEC-10 | policy, supervisor, evaluator | T02,T05,T06,T09,T10,T11; fixture suite |

## Open questions before implementation
Confirm graduation/exam schedule and weekly time, available RAM and storage, Linux/WSL2 access, faculty mentorship, chosen model provider or local-only policy, public data license, and intended public repository license. Confirm whether first milestone is ML accuracy/latency or algorithm performance. These do not block documentation; they do block claims of production readiness. All ADRs proposed, not accepted by a team. Update status to ACCEPTED only after review.
