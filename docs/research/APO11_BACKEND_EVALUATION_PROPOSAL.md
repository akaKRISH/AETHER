# APO-11 Backend Research-Quality Evaluation Proposal

**Status:** Historical proposal; superseded as the active wire contract by [`contracts/apo11/quality-evaluation.schema.json`](../../contracts/apo11/quality-evaluation.schema.json). The implemented structural-only slice and its limits are recorded in [`APO11_STRUCTURAL_EVALUATION.md`](APO11_STRUCTURAL_EVALUATION.md).

## Recommendation

Implement a bounded structural evaluator around existing content-addressed AETHER artifacts. Keep three evidence classes separate in both storage and display:

1. **Structural checks:** deterministic Rust checks for schemas, references, provenance fields, receipts, declared counterevidence, budgets, manifests, and digests.
2. **Human semantic review:** separate, append-only adjudications against a pinned rubric, linked to exact source passages. Preserve conflicting decisions and abstentions.
3. **Independent scientific validation:** unavailable until a separately governed protocol, independent reviewer/custodian, and evaluation data are established. The structural evaluator cannot set this status.

Do not calculate an aggregate quality, intelligence, truth, or novelty score. Structural completeness says that required records and links exist; it does not establish that a claim is true, a source supports it, a method is appropriate, or a study is valid.

## Existing building blocks and boundaries

- `crates/aether-api/src/models/workflow_evaluation.rs` already pins a versioned specification, serializes deterministic reports, hashes content, persists reports in the model registry and artifact store, and checks fresh-process identities. Reuse this pattern; do not introduce another report registry or runner.
- `crates/aether-api/src/literature/knowledge.rs` provides typed graph entities, revision pins, exact passage references, source groups, contradiction relations, and explicit `proposed`, `reported`, `extracted`, `inferred`, `observed`, and `independently_verified` states. Graph structure and extraction status are not proof of truth.
- `crates/aether-store/src/model.rs` and `crates/aether-store/src/evidence.rs` provide claim, evidence, contradiction, replay, and trusted-verification records. Trusted verification remains owned by the existing campaign protocol.
- APO-07/08/09/10 remain the skill registry, closed execution kernel, workflow orchestrator, and workflow-contract evaluator. APO-11 must not duplicate or bypass them. APO-10 reports are inputs with integrity/provenance, not scientific ground truth.
- The API uses authenticated loopback routes under `/v1`, an 8 KiB default request-body limit, and one local bearer token. That token authorizes local API access; it does not identify independent reviewers or establish their independence.

## Proposed module ownership

After interface approval, keep the first implementation in existing crates:

- `crates/aether-core/src/research_quality.rs`: version pins, artifact references, dimensions, assessments, budget types, and strict validation. No I/O.
- `crates/aether-engine/src/research_quality.rs`: deterministic structural checks over validated typed inputs. No database, network, model, or filesystem access.
- `crates/aether-api/src/models/research_quality.rs`: resolve local artifact references, verify hashes, call the engine, and persist immutable report bytes using the existing Lab/model-registry/artifact patterns.
- `crates/aether-api/src/http.rs` and `crates/aether-api/src/main.rs`: approved authenticated API and CLI adapters only.
- Rust tests stay with the owning modules. No frontend modification is proposed by Codex.

Do not add a crate, external evaluator, model-as-judge, provider integration, or new execution path. Keep human adjudication persistence out of the initial structural slice until reviewer identity and audit ownership are approved.

## Proposed typed contract

Use `#[serde(deny_unknown_fields)]` on externally deserialized structs/enums. Pin both identifier and content digest for every benchmark, rubric, and input artifact.

```rust
struct VersionPin { id: String, version: String, sha256: Sha256 }
struct ArtifactRef { kind: ArtifactKind, sha256: Sha256, pointer: Option<String> }
struct ResearchBundle { schema: String, artifacts: Vec<ArtifactRef> }
struct EvaluationBudget { max_checks: u32, max_input_bytes: u64, max_graph_nodes: u32 }

enum Dimension {
    ClaimSourceTraceability, EvidenceProvenance, CounterevidenceHandling,
    HypothesisFalsifiability, ExperimentDesign, MethodologicalCompleteness,
    UncertaintyAndAbstention, Reproducibility,
}
enum CheckOutcome { Pass, Fail, Inconclusive, Unavailable, NotApplicable }
struct StructuralCheck { id: String, outcome: CheckOutcome, subject: ArtifactRef,
                         evidence: Vec<ArtifactRef>, reason_code: String }
struct DimensionAssessment { dimension: Dimension, outcome: CheckOutcome,
                             checks: Vec<StructuralCheck> }
struct EvaluationReport { schema: String, benchmark: VersionPin, rubric: VersionPin,
                          bundle_sha256: Sha256, evaluator_sha256: Sha256,
                          configuration_sha256: Sha256,
                          dimensions: Vec<DimensionAssessment>, review_status: ReviewStatus,
                          independent_validation: IndependentValidationStatus,
                          limitations: Vec<String> }
```

These are proposed names, not committed API types. Hash strings must be exactly 64 lowercase hex digits. Bound vectors and strings before allocation/serialization. Sort artifact references, dimensions, and checks by stable keys before hashing. The report digest covers the deterministic serialized report payload without a digest field; the response envelope carries the digest. Pin the serialization/schema version and preserve old readers.

### Separate human adjudication record (future, gated)

```rust
struct HumanAdjudication {
    schema: String,
    report_sha256: Sha256,
    rubric: VersionPin,
    dimension: Dimension,
    verdict: HumanVerdict,       // supports | contradicts | insufficient | abstain
    evidence: Vec<ArtifactRef>,  // exact passage/artifact references
    reviewer_principal: String,  // supplied by approved auth context, never request text
    rationale: Option<String>,
}
enum ReviewerDisagreement { Open { adjudication_sha256s: Vec<Sha256> },
                            Resolved { resolution_sha256: Sha256 } }
```

Store each record content-addressed and append-only. Do not average ordinal judgments or silently select a winner. A disagreement remains visible until a separately authorized resolution cites its basis. A reviewer must not change a structural check or trusted campaign verdict. The current single bearer token supplies no distinct reviewer principal, so no human-review write endpoint is included in the initial API proposal.

### Evaluation lifecycle

- `EvaluationRun` is an execution record; its states (`completed`, `partial`, `failed`, `abstained`) describe evaluator execution only.
- Dimension outcomes are categorical and per-check. Missing required evidence may be `fail` when the pinned rubric says it is required; unsupported artifact/schema or insufficient inputs are `unavailable` or `inconclusive`, with a reason. Neither status is converted into a score.
- `ReviewStatus` is `not_started`, `pending`, `complete`, `disagreement`, or `unavailable`.
- `IndependentValidationStatus` is fixed by the evaluator to `not_performed`. It cannot be supplied or upgraded by the request, model, skill, or frontend.
- Evidence references that resolve and pass digest checks remain `unverified` unless the existing trusted verifier says otherwise.

## Dimension contract: what automation may say

| Dimension | Deterministic checks may establish | They cannot establish |
| --- | --- | --- |
| Claim-to-source traceability | Claim/source IDs resolve; passage pointer and digest are valid; required links exist. | Passage entails or accurately supports the claim. |
| Provenance/completeness | Required origin, version, license/provenance, source hash, and evidence fields are present and internally consistent. | Source authenticity, lawful status beyond recorded metadata, or evidence quality. |
| Contradiction/counterevidence | Declared contradiction edges, review fields, and referenced counterevidence resolve; required review record exists. | That all contrary evidence was found, or that a recorded contradiction is substantively correct. Absence of an edge is not evidence of no contradiction. |
| Hypothesis falsifiability | Proposition, observable outcome, falsification condition, and metric fields are present and typed. | That the hypothesis is scientifically meaningful or genuinely falsifiable in its domain. |
| Experiment design | Typed variables, baseline/control declarations, budget, seed policy, data split, and termination fields satisfy the pinned schema and registered task constraints. | That the design is fair, sufficiently powered, or fit for its scientific question. |
| Methodological completeness | Required sample-size assumptions, analysis plan, confounder/limitation fields, and preregistration references are present. | Correctness of statistical assumptions or methodology. Do not claim power without justified assumptions. |
| Uncertainty/abstention | Epistemic statuses, uncertainty fields, abstention reason, and unverified state are preserved. | Calibration or epistemic correctness. |
| Reproducibility | Input/configuration/runner hashes, seeds, manifest fields, and available replay receipt identities validate. | Independent reproduction unless an authorized fresh replay was actually run. |

Each report records the exact check IDs and source references. `unavailable` is an explicit value, not zero or a negative result. Structural counts may be displayed as counts of checks by categorical outcome; they must not be labeled a quality score.

## Proposed API contract for Antigravity

Use the existing authenticated `/v1` prefix. The canonical handoff ledger contains a draft `/api/v1` route pair; this proposal recommends aligning its semantics to AETHER's existing prefix and content-addressed path convention. The route choice remains unapproved.

### Submit structural evaluation

`POST /v1/eval/quality`

Request `QualityEvaluationRequest`:

```json
{
  "schema": "aether.quality-evaluation-request/1",
  "benchmark": {"id": "aether-research-quality", "version": "1.0.0", "sha256": "<64 lowercase hex>"},
  "rubric": {"id": "aether-research-quality-structural", "version": "1.0.0", "sha256": "<64 lowercase hex>"},
  "bundle": {"schema": "aether.research-bundle/1", "artifacts": [
    {"kind": "knowledge_graph_export", "sha256": "<64 lowercase hex>", "pointer": null},
    {"kind": "hypothesis_proposals", "sha256": "<64 lowercase hex>", "pointer": null},
    {"kind": "experiment_designs", "sha256": "<64 lowercase hex>", "pointer": null},
    {"kind": "workflow_receipt", "sha256": "<64 lowercase hex>", "pointer": null}
  ]},
  "dimensions": ["claim_source_traceability", "evidence_provenance", "counterevidence_handling",
                 "hypothesis_falsifiability", "experiment_design", "methodological_completeness",
                 "uncertainty_and_abstention", "reproducibility"],
  "budget": {"max_checks": 256, "max_input_bytes": 1048576, "max_graph_nodes": 512}
}
```

This shape is illustrative; allow no caller-provided verdict, reviewer identity, independent-validation state, paths, URLs, code, or arbitrary executable expressions. The service resolves only local content-addressed artifacts and enforces hard server-side ceilings even when the client requests lower limits. Body limit follows the current 8 KiB API limit; bundle input reads need an independently enforced byte/node cap.

Response: `QualityEvaluationReceipt { schema, report_sha256, report }`. The report carries benchmark/rubric/evaluator/configuration identities, bundle digest, categorical per-dimension assessments, check-level references, `review_status: not_started`, `independent_validation: not_performed`, limitations, and overall evaluator-run outcome. There is no numeric aggregate.

### Browse and inspect reports

- `GET /v1/eval/reports` — bounded, authenticated list of report digest, status, creation-order metadata, and schema version. Apply a hard page cap and return a `truncated` indicator as existing registry APIs do.
- `GET /v1/eval/reports/{sha256}` — retrieve a content-addressed report after validating its stored bytes against the requested digest.

### Frontend contract

Antigravity can build a read-only Research Quality view from the report list and inspect endpoints. It should show: report digest and pinned versions; the eight separate categorical dimensions; check details and exact provenance links; `unavailable`/`inconclusive`/`abstained` as distinct states; human review as pending/not started; independent validation as not performed; and limitations. A submit control may call only the structural evaluation endpoint after approval. Do not show a “quality score,” verified badge, or independent-review badge from structural checks.

No human-adjudication write endpoint is proposed until the owner approves reviewer identity, auth semantics, append-only audit behavior, and whether one local operator can be labeled a human reviewer. No independent-validation write endpoint is proposed; that needs a separately controlled protocol and custodian.

## Input validation, limits, persistence, and failures

- Resolve every artifact digest locally, verify stored bytes and typed schema, and reject unknown artifact kinds, bad pointers, duplicate references, stale rubric/spec pins, digest mismatches, and cross-version ambiguity before evaluation.
- Use fixed evaluator code only. Source passage text and imported metadata remain inert untrusted data. No shell, Python, JavaScript, model, provider, cloud, network lookup, held-out data, or arbitrary skill execution occurs.
- Enforce body bytes (existing 8 KiB), artifact bytes, graph nodes/edges, references, check count, depth, report bytes, and response bytes with checked arithmetic before work. Return a budget-abstention/failure record only if the report itself can be safely persisted within its bound.
- Persist report payload and digest in the existing content-addressed artifact store and registry pattern. On reuse, verify both registry bytes and artifact bytes against the digest. Treat it as tamper-evident local storage, not signed proof or a secure boundary.
- Proposed errors: `400 invalid_request/unsupported_schema`, `401 unauthorized`, `404 artifact_or_report_not_found`, `409 digest_or_version_conflict`, and `503 storage_or_internal_failure`, matching existing API status conventions. Keep user-facing details bounded and avoid returning source text by default.
- Deterministic hashing uses a pinned serialization/schema version and stable ordering. Test the same fixture/config in a fresh process; compare exact report digests. Any schema change gets a new version; never mutate frozen APO-10/ECAR artifacts.

## Development-only validation plan

Use synthetic or existing development fixtures only. Proposed pure-engine cases:

- complete, incomplete, missing, stale, duplicate, and mismatched claim/source references;
- malformed provenance, missing source passage, and digest tampering;
- present, missing, duplicate, and malformed contradiction/counterevidence links;
- missing falsification criteria and non-measurable/unsupported field shapes (structural result only);
- missing controls, budget overflow, absent split, missing statistical-plan fields, and unknown task IDs;
- uncertainty omitted, explicit abstention, and unavailable input each remain distinct;
- reproducible manifests and mismatched receipt/replay identities;
- conflicting human adjudications remain separate, invalid reviewer records are rejected, and model/user payloads cannot set reviewer or independent-validation authority;
- deterministic report identity across fresh processes and changed digest on any pinned input/config mutation.

These tests establish implementation contract behavior only. They do not measure scientific quality, calibration, novelty, correctness, or discovery. Human domain experts and a separately custodied preregistered held-out protocol would still be needed for those claims.

## Approval gates and open decisions

1. Owner approval of the exact route prefix/path and request/response schemas before changing Rust API routes, public types, or frontend contracts.
2. Owner decision on whether the first implementation is structural-only, with human-review data types specified but write API deferred. This is the recommended sequence.
3. A reviewer-identity and disagreement-resolution policy before any human adjudication endpoint. The current bearer-token model does not establish reviewer identity or independence.
4. Independent custody/protocol remains a separate future requirement; it does not block structural engineering, but independent scientific validation must remain `not_performed`.
5. Antigravity acknowledgement of the read-only frontend consumption shape and status labels before implementing UI integration.

Until those gates are approved, this proposal is the deliverable; no backend or frontend implementation is started.

## Inspection basis

Inspected the task-scoped project context/status/resume/queue, coordination ledger and Codex journal; ADR-014–018 and ADR-021–023; existing graph/evidence store models; APO-09/08 models; APO-10 workflow evaluator and routes; and current API auth/body-limit/error conventions. Codebase-memory graph lookup was unavailable for the AETHER repository, so source inspection used the approved text-search fallback. No source files, frozen research artifacts, or shared ledger were modified.
