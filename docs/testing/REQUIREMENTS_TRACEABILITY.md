# Requirements traceability

## Athena evidence

| Requirement area | Observed evidence | Remaining gate |
|---|---|---|
| Local proposal provenance | Installed digest, cloud state, inference options and output measured on the physical GPU. [Inference qualification](../operations/ATHENA_INFERENCE.md) | Measured scope covers one installed quantization and one host. |
| Frozen objective and sources | Exact-source v4 protocol approved at its recorded digest. [Draft snapshot](../../research/results/athena-local-draft-report.json) · [Completed run](../../research/results/athena-local-campaign.json) | One bounded registered sorting task demonstrates this end-to-end path. |
| Replay, challenge and trusted verdict | Pinned executable replayed the original receipt in a new process, evaluated a distinct challenge and matched the frozen verifier rule. [Campaign report](../../research/results/athena-local-campaign.json) | Repetition on new tasks requires a separate frozen protocol. |
| Reviewed execution lifecycle | Windows admission and descendant cleanup fixture. [Containment record](../security/ATHENA_WORKER_CONTAINMENT.md) | Qualify restricted identities, filesystem, egress and OS resource ceilings separately. |
| Matched policy experiments | Identical outcome rows across two development runs and frozen primary contrast. [Diagnostic protocol](../research/ATHENA_ECAR_PROTOCOL.md) | Reproduce an independently reviewed held-out evaluation before efficacy claims. |
| Constrained discovery | Tested mask validation, budget and stagnation stops, deterministic lineage, independent scoring and stored artifact integrity. [DAE-01 checks](../../RESUME.md#dae-01-constrained-discovery) | Scope covers the reviewed 12-item fixture. |

The mapping carries forward 12 product IDs, 14 technical IDs, 10 security IDs and 5 Phase 1 criteria. The original test IDs refer to the planned checks. The implementation evidence below specifies the local checks that actually ran.

| Requirement | Verification | Primary concern |
|---|---|---|
| PRD-F01 | T01, T23 | Valid project/task |
| PRD-F02 | T02 | Approval binds freeze |
| PRD-F03 | T03, T06 | Baseline execution |
| PRD-F04 | T05, T06 | Claim provenance |
| PRD-F05 | T08, T09 | Stop, recover |
| PRD-F06 | T01, T15, T28 | Candidate proposals |
| PRD-F07 | T04, T15 | Budgets |
| PRD-F08 | T07, T16 | Independent verifier |
| PRD-F09 | T05, T27 | Complete export |
| PRD-F10 | T13, T14 | Web interface |
| PRD-F11 | T25 | Remote GPU |
| PRD-F12 | T26 | Task SDK |
| TRD-N01 | T03, T17 | Local CPU and default concurrency |
| TRD-N02 | T04, T08 | Atomic reservations |
| TRD-N03 | T09 | Cancellation time |
| TRD-N04 | T05 | Hash checks |
| TRD-N05 | T24 | API latency |
| TRD-N06 | T10, T19 | Runner trust boundary |
| TRD-N07 | T04, T08 | Idempotent recovery |
| TRD-N08 | T12 | Restore |
| TRD-N09 | T03, T08, T11 | Event coverage |
| TRD-N10 | T14 | Accessibility |
| TRD-N11 | T17 | Native implementation |
| TRD-N12 | T18, T19 | Worker contracts |
| TRD-N13 | T09, T11, T18, T21 | Process-level failures |
| TRD-N14 | T20 | Memory safety policy |
| SEC-01 | T28 | Untrusted instructions |
| SEC-02 | T10, T19 | Host and secrets |
| SEC-03 | T10 | Egress |
| SEC-04 | T09, T11 | Quotas |
| SEC-05 | T05, T06, T07, T19 | Eval integrity |
| SEC-06 | T10, T18 | Artifact validation |
| SEC-07 | T02, T28 | Scoped authorization |
| SEC-08 | T20, T29 | Supply chain |
| SEC-09 | T10, T29 | Data and logs |
| SEC-10 | T04, T09, T28 | Bounded autonomy |
| M1-01 | T17, T20 | Build and offline runtime |
| M1-02 | T23 | Exact metrics and typed errors |
| M1-03 | T23 | Fresh process flow |
| M1-04 | T05, T23 | State and integrity |
| M1-05 | T23 | Help and owner walkthrough |

## Implementation evidence

These rows map the authorized A–N work to runnable evidence and outstanding scope. They retain the acceptance criteria in the original mapping above.

| Milestone | Local implementation and evidence | Remaining scope |
|---|---|---|
| A · Native engine | [Core tests](../../crates/aether-core/src/lib.rs) and [native fixture integration](../../crates/aether-cli/tests/native_fixture.rs) exercise exact metrics, inputs, fresh-process replay and tampering | Implemented for the native fixture |
| B · Evidence | [Store](../../crates/aether-store/README.md) exercises schema reopen, whole-attempt transactions, claim dependencies, revocation and fencing | Broader load, migration and crash scenarios |
| C · ECAR | [Scheduler and simulation](../../crates/aether-engine/README.md) exercise stable ranking, budget and matched synthetic arms. The real campaign records three actions and a stop. | Matched end-to-end pilot and held-out comparison |
| D · Worker | [Security review](../security/SECURITY_REVIEW.md) records the reviewed subprocess, bounded outputs, denied malformed evidence, active cancellation and abrupt supervisor recovery | OS isolation qualification and broader process-tree fault coverage |
| E · AI runtime | [Planner](../../crates/aether-api/src/planning.rs) exercises deterministic proposals and strict accepted algorithm selection | Live local inference, persistent multi-role agent state and external providers |
| F · Literature | [Catalog](../../crates/aether-api/src/literature.rs), [archive](../../crates/aether-api/src/literature/archive.rs) and [platform tests](../../crates/aether-api/tests/platform.rs) cover provider fixtures, resumable pages, exact spans and authority. [Graph tests](../../crates/aether-api/src/literature/knowledge/tests.rs) cover alias resolution, version pinning, conflicts and bounded traversal. | Live provider and PDF qualification, plus representative full-text extraction |
| G · Discovery | [Native sorting benchmark](../research/BENCHMARK_RESULTS.md) measures three reviewed candidates against a common reference | Adaptive variation and selection, multiple task campaigns and broader lineage |
| H · Compute | [Runtime profile](../security/SECURITY_REVIEW.md) bounds local concurrency and records worker time | Queue breadth, memory/CPU accounting, inference profiling, GPU and remote execution |
| I · API | [Loopback contract](../architecture/API_DESIGN.md) and [integration tests](../../crates/aether-api/tests/platform.rs) exercise approval, campaign, event resume, artifact retrieval, repeated draft cancellation and the authenticated design review boundary. [Design tests](../../crates/aether-api/src/literature/knowledge/hypotheses/experiment_design/tests.rs) cover canonical digests, matched budgets, overflow and leakage controls. | Hosted authorization, request tracing and release latency benchmarks |
| J · Workspace | [Browser smoke](../../tests/browser-smoke.cjs) covers an authenticated campaign and actual evidence and reports at desktop and mobile sizes | Dedicated accessibility audit and finer agent/compute controls |
| K · Benchmarks | [Recorded development experiments](../research/BENCHMARK_RESULTS.md) separate the simulation, process timings and real campaign | Equivalent end-to-end A–E policies, statistical protocol and independent held-out run |
| L · Hardening | [Validation](TEST_RESULTS.md) records native Rust, frontend, browser and restore checks | Remaining portability, load and active-worker fault coverage |
| M · Developer experience | [Startup and recovery guide](../operations/LOCAL_WORKBENCH.md) covers local scripts, health, backups and pinned release builds | Executed Linux and hosted CI qualification |
| N · Learning | [Codebase tour](../../CODEBASE_TOUR.md) and [rebuild curriculum](../learning/REBUILD_AETHER_FROM_SCRATCH.md) connect lessons to implemented source | Learner submissions and independent rebuild |

## Prometheus evidence

| Scope | Implementation evidence |
|---|---|
| Local correctness and admission | [Engineering audit](../engineering/PROMETHEUS_AUDIT.md) and [current tests](TEST_RESULTS.md) |
| Reviewed task and negative results | [Frozen taskpack](../../taskpacks/example-max-subarray.json) and [release report](../../research/results/prometheus-max-subarray.json) |
| Proposal boundary and normalized research sources | [Planner adapters](../operations/MODEL_PROVIDERS.md) and [literature adapters](../research/LITERATURE_ADAPTERS.md) |
| Local model inspection and development evaluation | [Metadata parser and fixtures](../../crates/aether-api/src/models/metadata.rs), [resource gate and trusted scoring](../../crates/aether-api/src/models/evaluation.rs) and [platform tests](../../crates/aether-api/tests/platform.rs) |
| Adapter-training feasibility | [Dataset validation](../../crates/aether-api/src/models/training/dataset.rs), [synthetic backend checks](../../crates/aether-api/src/models/training/backend.rs) and [platform tests](../../crates/aether-api/tests/platform.rs) |
| APO-06B dataset qualification | [Native dataset builder](../../crates/aether-api/src/models/datasets.rs), [reviewed recipes and provenance](../../crates/aether-api/src/models/datasets/cases.rs), [split validator](../../crates/aether-api/src/models/training/dataset.rs), [platform and browser tests](../../crates/aether-api/tests/platform.rs) · [ADR-020](../decisions/ADR-020.md) |
| Search, policy and performance evidence | [Development benchmarks](../research/BENCHMARK_RESULTS.md) and [simulated pilot](../research/PROMETHEUS_POLICY_PILOT.md) |
| UI, recovery and reproducibility | [Browser smoke](../../tests/browser-smoke.cjs) and [security review](../security/SECURITY_REVIEW.md) |
| Readiness and learning | [Remaining gates](../../NEXT_ACTIONS.md) and [Prometheus curriculum](../learning/IMPLEMENTATION_PROMETHEUS.md) |

Sources: [PRD](../product/FUNCTIONAL_REQUIREMENTS.md), [v0.2 technical requirements](../archive/v0.2/AETHER/04_TRD_SRS.md), [threat model](../security/THREAT_MODEL.md), [M1 acceptance](../roadmap/MILESTONE_01_NATIVE_FIXTURE.md) and [test plan](TEST_PLAN.md).
