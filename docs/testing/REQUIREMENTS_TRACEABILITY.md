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

## APO-07 research skill requirements

| Requirement | Verification | Evidence and limit |
|---|---|---|
| APO-07-01 · immutable identity | `same_skill_version_cannot_be_republished_with_changed_content`, registry integrity checks | Definition version is pinned by SHA-256; registry and content-addressed artifact must agree. |
| APO-07-02 · strict typed contract | `typed_schemas_reject_missing_unknown_and_wrongly_typed_values` | Bounded input/output schema validation; `/1` remains inert. |
| APO-07-03 · default-deny permissions | `imported_malicious_instructions_stay_inert_and_cannot_escalate_permissions` | Closed capability enum; imported declarations require local review and exact CLI approval. APO-09 execution is limited to the reviewed Rust operation allowlist. |
| APO-07-04 · safe composition and budgets | `dependency_validation_rejects_cycles_and_unpinned_missing_versions`, `budgets_use_checked_arithmetic_and_reject_overflow` | Pinned acyclic dependencies and checked aggregate limits; composition remains preflight-only. |
| APO-07-05 · reproducible invocation | `invocation_replay_is_deterministic_and_never_executes` | Content-addressed request and approval provenance; evidence references are shape-checked, not independently verified. |
| APO-07-06 · protected discovery | `authenticated_loopback_api_runs_a_campaign_and_serves_evidence`, Research OS browser smoke | Bearer-protected read/compose/preflight endpoints and read-only UI; review/approval remain local CLI operations. |

## APO-09 bounded skill execution requirements

| Requirement | Verification | Evidence and limit |
|---|---|---|
| APO-09-01 · explicit operation allowlist and immutable identity | `native_execution_is_approved_pinned_bounded_and_replayable`, `modified_operation_contract_and_stale_approval_fail_closed` | `/2` operation IDs map to Rust functions; `/1` definitions stay inert; execution pins ID, version and canonical digest. |
| APO-09-02 · default-deny capability intersection | `execution_intersects_permissions_and_keeps_evidence_unverified`, `execution_rejects_unpinned_versions_revocation_bad_input_and_limits` | Effective permissions are bounded by the operation-supported, declared and operator-approved sets. Local approvals are not signed credentials. |
| APO-09-03 · bounded composition, inputs, outputs and cancellation | `dependency_cycles_and_depth_limits_are_enforced_before_dispatch`, `oversized_output_and_tampered_receipts_fail_closed` | One root operation, depth at most eight, 8 KiB input and per-skill output capped at 256 KiB (1 MiB kernel ceiling); cancellation is cooperative and cannot preempt synchronous native code. |
| APO-09-04 · receipt integrity and deterministic replay | `native_execution_is_approved_pinned_bounded_and_replayable`, `oversized_output_and_tampered_receipts_fail_closed` | Deterministic content-addressed receipts bind skill, dependency, approval, operation, input/output and budget digests. |
| APO-09-05 · instructions and evidence remain untrusted | `imported_malicious_instruction_text_never_changes_the_native_operation`, `execution_intersects_permissions_and_keeps_evidence_unverified`, `execution_fails_closed_when_independent_verification_is_required` | Instruction strings are never dispatched; evidence refs receive structural checks only and remain unverified. Execution refuses contracts that require independent verification. |
| APO-09-06 · authenticated API boundary | `authenticated_loopback_api_runs_a_campaign_and_serves_evidence` | Execution and receipt-list routes are behind existing bearer authorization; no API approval or revocation route exists. |

## APO-08 workflow orchestration requirements

| Requirement | Verification | Evidence and limit |
|---|---|---|
| APO-08-01 · immutable, content-addressed workflow identity | `built_in_workflows_are_immutable_dags_with_reproducible_digests` | Definitions and artifacts must match canonical SHA-256; changed content cannot reuse a version. |
| APO-08-02 · typed DAG structure and mapping validation | `graph_validation_rejects_cycles_missing_nodes_duplicates_and_excessive_depth`, `mapping_validation_rejects_incompatible_types_and_undeclared_dependencies` | Strict schemas, exact skill pins, dependency ancestry, topological order, cycle/depth limits and declared output shapes are checked before execution. |
| APO-08-03 · exact workflow and skill approval | `workflow_requires_approval_and_replays_through_kernel_with_unverified_provenance`, `revoked_workflow_and_skill_approvals_fail_closed_and_cancellation_is_cooperative` | Local approval binds workflow digest and skill approval receipt pins; stale or revoked approval fails closed. Local consent is not a signature. |
| APO-08-04 · no permission expansion or alternate executor | `mapping_validation_rejects_incompatible_types_and_undeclared_dependencies`, authenticated loopback workflow request | Workflow declarations have no capability or instruction fields; every node uses APO-09 and required, declared, approved and operation-supported capabilities are checked. |
| APO-08-05 · deterministic sequential budgets and outcomes | `aggregate_budget_and_failed_dependencies_preserve_partial_and_skipped_states`, `missing_local_model_report_yields_an_abstained_workflow_receipt`, `revoked_workflow_and_skill_approvals_fail_closed_and_cancellation_is_cooperative` | Ordered node receipts include budget exhaustion, failed dependency skipping, cooperative cancellation, partial completion and abstention. Native operations are not preempted. |
| APO-08-06 · replay, receipt integrity and epistemic status | `workflow_requires_approval_and_replays_through_kernel_with_unverified_provenance`, `oversized_inputs_modified_pins_and_tampered_execution_receipts_are_rejected` | Content-addressed receipts verify kernel receipts and output artifacts; evidence remains `unverified`, and replay does not prove research correctness. |
| APO-08-07 · authenticated API and read-only Research OS | `authenticated_loopback_api_runs_a_campaign_and_serves_evidence`, desktop/mobile browser smoke | Registry, definition, execution and receipt routes require bearer auth; the UI inspects definitions/receipts but has no approval control. |

## APO-10 end-to-end workflow evaluation requirements

| Requirement | Verification | Evidence and limit |
|---|---|---|
| APO-10-01 · immutable benchmark and controlled fixtures | `specification_is_versioned_with_fixed_scenarios_and_metrics`, `workflow_benchmark_replays_identically_in_fresh_cli_processes` | Version 1.0.0 pins the specification and fixture digests; nine local development cases are stored in the content-addressed report. |
| APO-10-02 · workflow contract, outcomes, ordering and budget | `workflow_benchmark_replays_identically_in_fresh_cli_processes`, authenticated workflow benchmark API platform test | Nine expected outcomes/orders, typed-input checks, applicable output checks, aggregate budgets and dependent-node skipping passed. |
| APO-10-03 · adversarial mutations and failure propagation | `workflow_benchmark_replays_identically_in_fresh_cli_processes` | Seven digest, pin, permission, dependency, evidence-reference and receipt-integrity cases were rejected as expected. |
| APO-10-04 · fresh-process replay and receipt integrity | `workflow_benchmark_replays_identically_in_fresh_cli_processes` | Two fresh processes matched canonical worker-report and nine workflow receipt digests; this is deterministic behavior evidence, not scientific validation. |
| APO-10-05 · evidence status and scientific metric separation | `workflow_benchmark_replays_identically_in_fresh_cli_processes`, desktop/mobile Research OS browser smoke | Evidence remained unverified; scientific-quality measures are null and marked unavailable. Independent human review and separately custodied evaluation remain open. |
| APO-10-06 · authenticated discovery, execution and report inspection | authenticated workflow benchmark API platform test, desktop/mobile Research OS browser smoke | Existing bearer middleware protects API routes; interface is an operator-triggered fixture evaluation and read-only report inspector. No held-out execution or external/model calls occur. |
