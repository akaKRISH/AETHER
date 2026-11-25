# Editable architecture diagrams

| Diagram | What it documents |
|---|---|
| [System context](system-context.mmd) | Owner, laboratory, reviewed tasks and optional providers |
| [Container architecture](containers.mmd) | Trusted process, worker isolation and data |
| [Components](components.mmd) | Protocol, scheduler, evaluator and store responsibilities |
| [Crate dependencies](crate-dependencies.mmd) | Inward Rust dependencies and composition roots |
| [ECAR lifecycle](ecar-lifecycle.mmd) | Eligibility and budget before action ranking |
| [Experiment execution](experiment-execution.mmd) | Dispatch from owner approval through settlement |
| [Worker communication](worker-communication.mmd) | Versioned envelope, bounded file output and integrity |
| [Evidence graph](evidence-graph.mmd) | Claim dependencies and evidence links |
| [Entity relationships](entity-relationships.mmd) | Local persistence relationships |
| [Campaign state](campaign-state.mmd) | Approval, execution, pause and terminal outcomes |
| [Claim state](claim-state.mmd) | Preliminary support, terminal verdict and revocation |
| [Run lifecycle](run-lifecycle.mmd) | Queue entry, fenced attempt and terminal states |
| [Budget reservation and settlement](budget-ledger.mmd) | Atomic reservation and idempotent accounting |
| [Security trust boundaries](trust-boundaries.mmd) | Untrusted proposals and artifacts across controlled interfaces |
| [Sandbox](sandbox.mmd) | Reviewed execution and minimum actor capabilities |
| [Distributed workers](distributed-workers.mmd) | Future opt-in remote transfer and trusted outcome checks |
| [Deployment](deployment.mmd) | Phase 1, wider local release and future R2 services |
| [Failure recovery](failure-recovery.mmd) | Reconciliation and bounded retry |
| [Verification pipeline](verification-pipeline.mmd) | Independent evidence, protected evaluation and verdict |
| [User research journey](user-journey.mmd) | Owner approval, execution and evidence review |
| [CI/CD pipeline](ci-cd.mmd) | Validation and release authorization |

These diagrams describe proposed designs. Each editable source has a corresponding [rendered SVG](rendered/). See [architecture](../architecture/README.md) and [audit](../DOCUMENTATION_AUDIT.md). Original images and source files are in the [source archive](../archive/v0.2/AETHER/diagrams/).
