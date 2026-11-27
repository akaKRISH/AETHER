# APO11-003 — Backend Evaluation Architecture Red-Team Review

**Review status:** Design review complete; no source implementation or route decision made.
**Scope:** APO11 backend proposal, existing graph/evidence/report/API patterns, and human-review feasibility. Development artifacts only; no held-out data or scientific conclusions inspected.

## 1. Working environment verification

| Check | Observed result |
| --- | --- |
| Repository | `C:\Users\2k25c\PROJECTS\AETHER-apo11-codex` |
| Branch | `work/apo11-codex` |
| HEAD | `5a222b7213e124c8f2faf7757a52f4c9cf7c4d33` |
| Proposal commit | `c6eb8c87f2c2c3d6316f4799b975e598674b7ef9` exists; parent is `d12fda0cf74f765e9fa33285c45ea79e4ab4bd7f` |
| Journal commit | `5a222b7213e124c8f2faf7757a52f4c9cf7c4d33` exists; parent is `c6eb8c87f2c2c3d6316f4799b975e598674b7ef9` |
| Ancestry | `d12fda0cf74f765e9fa33285c45ea79e4ab4bd7f` is an ancestor of HEAD; verified with `git merge-base --is-ancestor` (exit 0). |
| Working tree | Clean; `git status --short --branch` returned only `## work/apo11-codex`. |

## 2. Executive assessment

The proposal has the right governing boundary: structural checks, human semantic assessment, and independent scientific validation must remain separate; no overall “quality” or truth score should be emitted; and the evaluator must not promote evidence to verified. The existing graph, campaign verifier, artifact store, and APO-10 report path provide useful pieces.

It is **not implementation-ready**. The largest gaps are (1) the proposed `ArtifactRef` cannot preserve the distinct source, passage, provenance, and epistemic identities required for evidence traceability; (2) `serde_json::to_vec` is deterministic for the current typed shapes but is not a declared interoperable canonical JSON algorithm; and (3) a single local bearer token and the literal `local_operator` approval label cannot establish reviewer identity or independence. The route prefix also remains unresolved between the live `/v1` router and the proposed `/api/v1` shared-ledger draft.

All eight dimensions should be labeled **structural completeness** in machine reports. None establishes evidence quality or scientific truth. Human semantic assessments are a separate record type. Experimental support remains a result of the existing trusted campaign verifier under its exact frozen protocol, not a conclusion of APO-11.

## 3. Findings by severity

### BLOCKER — Human-review writes cannot attribute a reviewer today

The API state holds one configured bearer token and `authorize` checks only whether the supplied token matches it (`crates/aether-api/src/http.rs:21-33,157-189`). There is no authenticated user/principal or role context in the API code. Workflow approval receipts use the literal `reviewer: "local_operator"`, and validation checks that same literal (`crates/aether-api/src/models/workflows.rs:111-126,822-855`). This is an operator label, not a person identity, signature, or independent-review proof.

**Required correction:** Keep human-adjudication write endpoints disabled. A future endpoint must receive reviewer identity from an approved authentication context, never from request JSON. If AETHER continues to use one local token, the most it can record is a local operator attestation. It must not label that attestation independent or externally authenticated. Owner approval is required for identity, role separation, conflict-of-interest, and correction/appeal policy.

### HIGH — Evidence references collapse distinct identities and statuses

The proposed `ArtifactRef { kind, sha256, pointer }` can identify a stored artifact and a JSON pointer, but it does not identify the original source version, exact passage representation/span, provenance chain, source group, or epistemic status. It also risks treating “the pointer resolves” as “evidence supports the claim.” Current graph `Source::Passage` has paper/version/document/source hash/URL/license/start/end/section, while `Revision` separately has revision digest, source group, and optional content digest (`crates/aether-api/src/literature/knowledge.rs:85-165`). Campaign evidence is a different relation to a metric or completed attempt artifact (`crates/aether-store/src/model.rs:215-233`, `crates/aether-store/src/evidence.rs:62-100`).

**Required correction:** Add a typed evidence reference that contains separate source, passage, provenance, and relation identities; separate structural resolution from human assessment and trusted experimental verification. Preserve exact source bytes and exact locations. Do not replace or normalize the original source. A JSON pointer may locate a field inside a pinned AETHER artifact, but it is not a passage identity.

### HIGH — The hash contract is underspecified for long-term replay

APO-10 calls `serde_json::to_vec` from a function named `canonical` and hashes those bytes (`crates/aether-api/src/models/workflow_evaluation.rs:299-307`). Its current report has no wall-clock timestamp; `run` launches two fresh processes of the same executable and compares their report and receipt digests (`:707-792`). This is useful same-build replay evidence, but the proposal does not pin a canonicalization standard or number/Unicode rules. The workspace pins `serde_json = "1"` and `sha2 = "0.10"`; the resolved `serde_json` package has no `indexmap` dependency in its lockfile entry, and no JCS or Unicode-normalization dependency is declared (`Cargo.toml:23-30`, `Cargo.lock:1085-1118`). Struct field order and current default map ordering are not a cross-language canonicalization contract.

**Required correction:** Version the byte encoding explicitly. Either adopt a reviewed RFC 8785/JCS implementation with published test vectors, or define a typed AETHER canonical form and make no claim of cross-language JCS compatibility. For the latter, reject arbitrary floating-point numbers in hashed report fields; use integers/fixed-scale integers or canonical decimal strings, sort all set-like vectors, pin map ordering, and add independent known-answer vectors. State that no Unicode normalization is applied to source bytes; hashes bind exact UTF-8 bytes. Put run timestamps, database row IDs, filesystem paths, PIDs, and elapsed time outside the canonical report payload.

### HIGH — The eight “dimensions” do not yet have executable rules

The proposal gives examples of what each dimension might check, but not the exact rubric version, required artifact schemas, per-rule IDs, outcome precedence, or missing/unsupported-input mapping. Without those, two evaluator versions can emit different states for the same input while both claim the same rubric version.

**Required correction:** Give each check a stable ID and version, typed inputs, deterministic predicate, and explicit outcome. Pin the complete rubric digest. Keep dimension status categorical and show every check. Do not infer a dimension-level pass from a subset of available checks. An unsupported schema or absent optional artifact is `unavailable`; a required field missing under the pinned rubric is `fail`; a truncated or insufficient input is `inconclusive`/`abstained` according to the pinned rule. `not_applicable` must come from the rubric’s declared scope, not from a missing artifact.

### HIGH — Request limits do not constrain referenced artifacts or report size

The router sets an 8 KiB request-body cap (`crates/aether-api/src/http.rs:131`), but a request can name stored artifacts. `Lab::artifact` returns a fully materialized `Vec<u8>` (`crates/aether-api/src/lab.rs:626-631`); the store accepts artifact blobs up to 8 MiB each (`crates/aether-store/src/execution.rs:4-5,303-317`). The proposed request budget values are caller-provided examples and the proposal does not specify a server-side aggregate cap, report cap, concurrency cap, or pre-read sizing policy.

**Required correction:** Enforce fixed server ceilings independent of client values: maximum references, per-artifact bytes, aggregate bytes, graph nodes/edges, JSON nesting, rules/check count, report bytes, and response bytes. Check stored sizes before reading content. Add bounded concurrency/backpressure for evaluation. A limit must be charged before allocation/work, with checked arithmetic. If a safe report cannot be emitted after exhaustion, return a bounded typed error rather than constructing an oversized partial report.

### MEDIUM — Report and registry persistence spans separate SQLite stores

`Lab` stores campaign artifacts through `laboratory.sqlite3` and opens a separate `model-lab.sqlite3` for the model registry (`crates/aether-api/src/lab.rs:215-239,648-654`). APO-10 writes artifact bytes and then registry data, while the registry helper uses `ON CONFLICT ... DO UPDATE` (`crates/aether-api/src/models/workflow_evaluation.rs:780-792`, `crates/aether-api/src/models.rs:247-276`). There is no cross-database atomic transaction, and the upsert can replace a record at an existing content key if a caller uses it incorrectly.

**Required correction:** Keep the artifact blob as the source of report bytes. Write the blob first, then insert the registry reference in one immediate SQLite transaction using insert-if-absent semantics; on an existing digest, read back and verify exact bytes instead of overwriting. A crash may leave an orphan blob, but must never expose a registry row pointing to absent or mismatched content. Make retrieval verify both row and blob against the requested digest. Document that local DB/file owners can still alter both stores; this is tamper-evident integrity, not a signature or secure boundary.

### MEDIUM — Graph truncation and evolving revisions can look complete

`graph_export` is a snapshot with a graph digest and a SQLite read transaction, but it deliberately caps graph traversal and reports a `truncated` flag (`crates/aether-api/src/literature/knowledge.rs:524-652`). It exports current node revisions plus only selected history. A root ID is stable while its current revision may change. A report that references only the root ID or receives a truncated export cannot claim graph-wide completeness.

**Required correction:** Pin the full graph-export digest, its root ID, all node/revision digests used, and every source/passage digest. Reject or mark the affected dimension inconclusive when `truncated=true`. Do not re-query mutable “latest” graph state during replay; replay the pinned export artifact.

### MEDIUM — API error mapping can misclassify failures

`ApiError` distinguishes input, missing, conflict, integrity, and service failures (`crates/aether-api/src/http.rs:37-64`), but APO-10 handlers map broad evaluator errors to `Error::Input` or `Error::Storage` (`:601-627`). Reusing that pattern could return malformed-input `400` for a stale digest or implementation/storage fault, or leak details through the `Input` message path.

**Required correction:** Add typed quality errors and a deliberate HTTP mapping: invalid request/schema `400`; absent requested artifact/report `404`; digest/version conflict `409`; exhausted server capacity `429` or `503`; internal/storage failure `503`. Return stable reason codes and bounded safe details only. Do not echo source text, request payloads, paths, SQL, or raw parser errors.

### MEDIUM — Human review history is described as append-only without a persistence contract

`model_registry` is an upsert table; it is not an append-only review log. The campaign store has immutable triggers for event and verification rows (`crates/aether-store/src/001_initial.sql:130-154`), but those guarantees do not automatically apply to review records or a second database.

**Required correction:** If human review is later approved, use separate immutable review-event records with update/delete rejection, stable sequence, record digest, and `supersedes`/`appeals` links. Corrections add a new event; they never overwrite a prior decision. A hash chain can reveal some edits but is not protection against a local database owner who can rewrite the full chain. Do not call local records signed proof.

### MEDIUM — Caller-selectable dimensions permit selective reporting

The proposed request accepts a `dimensions` list. A caller could omit failed or unavailable dimensions and show a partial report as a complete assessment.

**Required correction:** For v1, always emit all eight dimensions. If future scoped evaluation is needed, pin the scope in the rubric and mark omitted dimensions `not_requested`; the report must never label such a subset a complete evaluation.

### LOW — “Quality” wording can imply more than is measured

The endpoint and UI may say “quality” while the automated engine checks structural completeness. Keep `assessment_class: structural_completeness` on every dimension and describe aggregate counts as counts of checks only. Do not show a score, verified badge, or quality pass/fail based on structure alone.

## 4. Eight-dimension structural contract

The evaluator must not grade evidence quality or scientific truth. These eight dimension results report **structural completeness only**. Any substantive semantic assessment belongs in a separate human record; independent scientific validation remains outside APO-11 automation.

For every dimension, emit:

```text
DimensionResult {
  dimension_id, dimension_version, assessment_class: "structural_completeness",
  status, checks: [CheckResult { rule_id, rule_version, status,
                                subject_refs, evidence_refs, reason_code }]
}
```

The status set is `pass | fail | inconclusive | unavailable | not_applicable | not_requested`. `not_applicable` requires a pinned rubric declaration. A required field missing under the pinned rule is a structural `fail`; unavailable source bytes or an unsupported schema is `unavailable`; a truncated graph or incomplete input is `inconclusive`; a budget cap reached before checks complete is a run-level `abstained` with per-check `unavailable` and reason `budget_exhausted_before_check`. `EvaluationRun.status` describes whether the evaluator executed (`completed | partial | failed | abstained`), never whether the research is good.

### 4.1 Claim-to-source traceability

- **Definition/class:** Structural linkage completeness, not support quality or entailment.
- **Inputs:** Pinned graph export; claim ID and exact claim revision; relation edge; typed source identity; passage identity; stored source/representation digest.
- **Algorithm:** Verify export digest and `truncated=false`; resolve claim and referenced source/revision; verify edge endpoints/revisions and allowed relation; verify the passage span lies within the exact pinned representation and its passage digest matches. Check required links according to rubric. Do not compare claim meaning to passage text.
- **Output:** `claim_exists`, `claim_revision_matches`, `source_exists`, `edge_valid`, `passage_span_valid`, `passage_digest_matches`, each as a check result with exact refs.
- **Failure:** Required claim/source/edge absent or invalid span/digest -> `fail`; malformed external request -> request error, no report.
- **Unavailable/inconclusive:** Source representation not archived or unsupported passage representation -> `unavailable`; truncated export/history -> `inconclusive`.
- **Determinism:** Sort by claim ID, claim revision digest, relation, source digest, passage digest; use pinned graph export, not mutable latest state.
- **Ceiling:** Establishes resolvable links only; not that a cited passage supports, contradicts, or accurately represents the claim.

### 4.2 Evidence provenance and completeness

- **Definition/class:** Presence and internal consistency of provenance fields; not provenance authenticity or evidence quality.
- **Inputs:** Evidence reference, source identity/version/digest, original bytes or archived representation, capture/ingestion provenance, source group, license record, extraction tool/version where applicable.
- **Algorithm:** Validate source ID normalization by its versioned type; verify source bytes against source digest; verify the exact representation digest and locator; check required provenance fields and source-group references. For text spans, check UTF-8 byte boundaries and `0 <= start < end <= byte_len`, then recompute passage digest. For extracted PDF text, bind both original PDF digest and extracted-text digest plus extractor/version/page locator.
- **Output:** `source_digest_valid`, `representation_bound`, `passage_location_valid`, `ingestion_provenance_present`, `license_record_present`, `source_group_resolves`. Do not produce an evidence-quality score.
- **Failure:** A required provenance field is absent or digest/locator conflicts -> `fail`.
- **Unavailable/inconclusive:** Source bytes or extraction manifest unavailable -> `unavailable`; unsupported locator/unit -> `inconclusive`.
- **Determinism:** Exact source bytes and extraction recipe are immutable inputs; order references by stable source/passage digest; timestamps are provenance observations but are not substituted for source identity.
- **Ceiling:** Metadata presence does not prove source authenticity, rights, independence, or relevance.

### 4.3 Contradiction and counterevidence handling

- **Definition/class:** Structural completeness of declared contradiction checks, not discovery or adjudication of all counterevidence.
- **Inputs:** Pinned claim revision, declared search/review scope, contradiction/counterevidence edges, exact evidence refs, and an explicit review outcome (`found | none_in_scope | not_assessed | abstained`) with scope/provenance.
- **Algorithm:** Resolve each declared contrary edge and evidence reference; validate revision pins; require a review record for claims whose rubric requires counterevidence review. An empty contradiction-edge set is never interpreted as “no counterevidence.”
- **Output:** `declared_edges_resolve`, `review_scope_present`, `review_outcome_present`, `not_assessed_reason_present`.
- **Failure:** Required review record/edge is absent or malformed -> `fail` under the pinned rubric.
- **Unavailable/inconclusive:** Search scope/source coverage is not recorded or bounded corpus is incomplete -> `unavailable`/`inconclusive`, never “no contradictions.”
- **Determinism:** Evaluate only pinned evidence/review artifacts; stable sort by claim and evidence identity.
- **Ceiling:** Cannot establish literature-search recall, contradiction truth, or completeness of the evidence universe.

### 4.4 Hypothesis falsifiability

- **Definition/class:** Presence and type validity of falsification fields, not substantive falsifiability.
- **Inputs:** Versioned hypothesis proposal with proposition, observable outcome, falsification condition, metric, threshold/decision rule, assumptions, alternatives, limitations, and evidence refs.
- **Algorithm:** Parse the pinned schema; require non-empty bounded fields; check metric identifiers and units resolve to registered definitions; check decision-rule fields are well-typed and thresholds are in supported numeric domains. Do not judge whether the proposition is coherent or falsifiable in the real domain.
- **Output:** One result per required field/reference, e.g. `proposition_present`, `falsification_rule_typed`, `metric_registered`, `threshold_valid`.
- **Failure:** Required field absent or invalid typed reference -> `fail`.
- **Unavailable/inconclusive:** Unsupported hypothesis version or unregistered domain metric -> `unavailable`; unresolved semantics -> `unavailable`, never pass.
- **Determinism:** Use pinned schema and metric registry digest; forbid free-form heuristic scoring.
- **Ceiling:** A syntactically explicit falsifier may still be impossible, irrelevant, or unfalsifiable in practice.

### 4.5 Experiment design

- **Definition/class:** Structural conformance to the declared design schema and registered task constraints, not design quality or fairness in the scientific sense.
- **Inputs:** Pinned design, hypothesis reference, registered task ID/version, variables, baseline/control declarations, data split identities, seed policy, resource budget, termination rules, and any comparison-arm registry pins.
- **Algorithm:** Resolve every task/metric/arm pin; validate variable roles and control declarations; checked-add per-arm and aggregate resource limits; verify dataset identity splits do not reuse a held-out identity in development; verify oracle phase/order constraints from the frozen design. Do not execute tasks or inspect held-out data.
- **Output:** `hypothesis_pin_valid`, `task_registered`, `variables_typed`, `baseline_declared`, `budget_arithmetic_valid`, `split_identity_disjoint`, `seed_policy_present`, `termination_present`.
- **Failure:** Invalid/unknown task, overflow, invalid role, or pinned split overlap -> `fail`/request rejection according to contract.
- **Unavailable/inconclusive:** No registry contract for a field or no split identities available -> `unavailable`; no inference of a valid design.
- **Determinism:** All registries and datasets are content-pinned; sorting and checked integer arithmetic only; no current hardware-dependent estimates in the content-addressed result unless separately pinned as an input.
- **Ceiling:** Cannot establish fair comparison, sufficient power, construct validity, or domain appropriateness.

### 4.6 Methodological completeness

- **Definition/class:** Required-method-field presence and linkage, not validity of statistical assumptions.
- **Inputs:** Analysis plan with sample-size assumptions, outcome definitions, preregistered metrics, statistical method/version, multiplicity/missing-data handling, confounders, stopping criteria, and limitation records.
- **Algorithm:** Check required fields for the pinned study class; validate sample-size assumptions reference actual parameters; require justified assumptions when the plan makes a power claim; verify metric/statistics IDs resolve to pinned definitions; verify termination and exclusion rules are present.
- **Output:** Per-field completion checks; no computed “power passed” result unless a separately reviewed power calculator and assumptions are pinned.
- **Failure:** Required plan field missing/invalid -> `fail`.
- **Unavailable/inconclusive:** Unsupported analysis method, absent assumptions, or unpinned metrics -> `unavailable`/`inconclusive`; do not emit a power estimate.
- **Determinism:** Versioned checklist and registry pins; no semantic model; exact numeric representation.
- **Ceiling:** Completeness cannot validate statistical appropriateness or guarantee a valid inference.

### 4.7 Uncertainty and abstention

- **Definition/class:** Presence and preservation of epistemic labels, missingness, and abstention reason; not calibration.
- **Inputs:** Claim/hypothesis/evidence status, confidence representation where applicable, missing-result records, workflow abstention/failure receipts, and limitations.
- **Algorithm:** Validate allowed epistemic-status values; ensure unverified references remain unverified; check confidence bounds; require reason codes for abstentions and unavailable results; preserve negative and inconclusive states without coercing them into zero/false.
- **Output:** `epistemic_status_valid`, `confidence_in_range`, `abstention_reason_present`, `missingness_preserved`, `unverified_state_preserved`.
- **Failure:** Invalid status or a required abstention reason omitted -> `fail`.
- **Unavailable/inconclusive:** Unsupported confidence scale or absent source status -> `unavailable`; unknown truth remains explicitly unknown.
- **Determinism:** Fixed enums, integer basis-point confidence if present, stable reason codes; no timestamps in hashed report.
- **Ceiling:** Presence of uncertainty fields does not show calibrated or honest uncertainty.

### 4.8 Reproducibility

- **Definition/class:** Presence and integrity of replay inputs/receipts; not actual reproduction unless a replay was run.
- **Inputs:** Original artifact digest, protocol/rubric/configuration pins, evaluator build ID, seed policy/seeds, source and dataset pins, output receipt, and optional actual fresh-process replay receipt.
- **Algorithm:** Recompute available blob digests; validate all identity pins and schema versions; compare the stored replay result/receipt to the expected parent; distinguish `manifest_complete` from `replay_observed`. APO-11 itself performs no arbitrary experiment or external call.
- **Output:** Separate checks for `manifest_complete`, `input_digests_match`, `receipt_valid`, and `fresh_replay_match`; if no replay exists, replay check is `unavailable` or `not_applicable` per pinned rubric, not `pass`.
- **Failure:** Digest/receipt mismatch -> `fail`.
- **Unavailable/inconclusive:** No replay receipt or unsupported runner -> `unavailable`; only same-process record available -> `inconclusive` for independent replay.
- **Determinism:** Pin exact serialized inputs, configuration, executable identity, canonicalization version, and seed; compare exact bytes/digests in a fresh process.
- **Ceiling:** A matching receipt verifies the recorded computation contract, not scientific validity or generalizability.

## 5. Evidence integrity corrections

The following objects must not be represented by one generic `ArtifactRef`:

1. **Evidence reference:** A typed relationship from a claim/finding to an evidence object; stable local ID and relation (`supports`, `contradicts`, `derived_from`, etc.). It says only that a record points somewhere.
2. **Source identity:** Canonical source identifier/type, publication/version identity, exact source-byte digest, media type, and source-group identity. DOI/URL is an alias/locator; the content digest identifies the retrieved version.
3. **Passage identity:** Source digest plus representation digest, location unit, exact start/end offsets, optional page/section locator, and passage-byte digest. This binds the location to the exact source representation.
4. **Provenance:** Acquisition/import method, observed source locator, retrieval/ingestion event, license assertion, parser/extractor/version, and source-group lineage. Provenance records are assertions with their own evidence; their presence is not authentication.
5. **Structural validity:** A check receipt that all referenced identities resolve and hashes/offsets match. It is a machine result, not a verification verdict.
6. **Independent verification:** A reference to an authorized trusted verification receipt pinned to the frozen rule/protocol and evaluator; APO-11 may display/validate the reference but cannot create it.
7. **Human assessment:** A separate reviewer record linked to report/rubric/evidence; it can express support, contradiction, insufficiency, abstention, or disagreement but cannot set trusted verification.
8. **Experimentally supported finding:** A claim/result tied to a completed campaign, exact protocol/evaluator, and trusted verifier outcome. Describe it as verified under that protocol; do not generalize the record into universal truth.

Recommended separation:

```text
EvidenceReference { id, claim_revision, relation, source: SourceIdentity,
                    passage: PassageIdentity, provenance: ProvenanceRef }
StructuralIntegrity { reference_id, status, checks, evaluator/rule pin }
HumanAssessmentRef { adjudication_digest, reviewer_principal?, rubric_pin }
TrustedVerificationRef { campaign_id, protocol_digest, evaluator_digest,
                        claim_id, verdict, support_evidence_digests, replay_digest? }
```

The `reviewer_principal?` remains unavailable until identity is approved; it must never come from caller-controlled JSON. Existing `Source::Passage` is a strong starting point, but the APO11 contract must bind the extraction representation and exact span, and must carry epistemic status separately from the reference itself. Existing source groups and duplicate links are useful for avoiding false independence claims; group membership must not be scored as source quality. A graph export with `truncated=true` must not satisfy completeness.

## 6. Determinism and report identity

### Current behavior

- APO-10’s `canonical()` currently returns `serde_json::to_vec`; `digest()` applies SHA-256 to those bytes (`crates/aether-api/src/models/workflow_evaluation.rs:299-304`).
- The content-addressed report is hashed without embedding its own hash; the hash is returned in a wrapper (`:780-792`). This avoids self-reference and is a pattern worth keeping.
- APO-10 repeats the same fixture run in two fresh processes of the same executable, compares executable/spec identity and report/receipt digests (`:707-779`). This supports same-binary, same-host replay; it does not prove cross-version, cross-platform, or cross-language canonicalization.
- APO-10 report payload contains deterministic counts and fixture digests, not a wall-clock timestamp. List order uses SQLite `rowid`, which is presentation/insert order and must not enter a report digest (`:665-692`).

### Required hashed payload fields

Include: report schema; canonicalization ID/version; benchmark and rubric IDs, versions, and hashes; evaluator/build digest; configuration hash; pinned bundle digest and sorted artifact/source/passage/revision references; all eight dimensions and their pinned rule versions; sorted check results/status/reason codes; requested/scope declarations; deterministic integer budget limits and consumption; and explicit unavailable/abstention/limitation codes.

Exclude from the canonical payload: the report’s own digest, DB rowid, generated-at timestamp, PID, temp path, absolute file path, HTTP headers, request arrival/order, elapsed wall time, cache state, and page/list envelope. If an audit timestamp is later required, store it as a separate review/run event or outer storage metadata. Human-review decisions are separate immutable records; a structural report may cite their digests but must not acquire new timestamps or change its own digest when a human review is appended.

### Encoding rules

- Pin one canonicalization algorithm and version. Do not refer to plain `serde_json::to_vec` as canonical JSON without qualifying its exact behavior.
- Keep report structs closed (`deny_unknown_fields` on requests), avoid free-form `serde_json::Value` in hash-critical fields, and use sorted vectors for set semantics.
- Prefer integer and fixed-scale numeric types. If decimal values must be hashed, specify a canonical decimal encoding; reject NaN/infinity and normalize negative zero explicitly.
- Hash original source files as raw bytes. For text passage representations, preserve exact UTF-8 bytes and do not silently NFC/NFD-normalize them. Normalize identifiers only with a versioned identifier-specific rule, and retain the original spelling/alias.
- Use a domain-separated hash preimage such as `AETHER-QUALITY-REPORT\0v1\0 || canonical_payload`. Keep report digest outside the payload envelope.
- Add known-answer serialization/hash vectors and run them in separate processes. Test stable output across supported toolchains/platforms before promising cross-build replay. A report containing evaluator binary identity will intentionally differ when that binary changes; compare only reports with matching evaluator/configuration pins.

## 7. Security, resources, and persistence

### Authentication and authorization

- Existing protected API routes use one local bearer token; response caching is disabled (`http.rs:157-189`). This is suitable for local access control but not distinct reviewers, roles, identity proof, or independence.
- Keep evaluation request auth at the existing boundary. No route body may grant a capability, set `verified`, set reviewer identity, or mark independent validation.
- Human review and any approval-affecting behavior remain disabled pending owner policy. A review is not execution approval and must not change a campaign verdict.

### Input and output bounds

- The current router limit is 8 KiB for request bodies (`http.rs:131`). Do not interpret this as an artifact bundle limit.
- `Lab::artifact` materializes full bytes; store writes cap an individual blob at 8 MiB (`lab.rs:626-631`, `aether-store/src/execution.rs:4-5,303-317`). Enforce aggregate bundle size and preflight content length before reads.
- Bound references, graph nodes/edges, revisions, source passages, per-artifact and aggregate bytes, JSON depth, rule count, output checks, serialized report bytes, and response bytes. Use checked arithmetic and fixed server ceilings. The client may request stricter limits only.
- Bound concurrent evaluation requests with a small admission semaphore and return a typed capacity result. `spawn_blocking` alone does not define a per-feature concurrency limit.
- Keep source passage text out of report bodies by default. Return digests, locations, and reason codes. Never log bearer tokens or full imported documents. The current generic storage error response is bounded; avoid forwarding raw parser/SQLite details.

### Transactions and integrity

- The artifact blob store and model registry live in distinct local stores. Atomicity across them is unavailable. Use blob-first, registry-reference-second; store blobs by digest and verify collision/idempotent replay; create a registry reference only after blob write succeeds. An orphan blob after a crash is acceptable; a visible dangling report is not.
- Make report registry inserts immutable by digest. The existing `Registry::save` uses UPSERT, so do not call it blindly for reports. On a duplicate `(kind, digest)`, read/verify exact bytes and return the existing record; reject mismatches.
- Apply transaction boundaries to the report registry insert, but do not hold a SQLite transaction open while parsing large artifacts or executing checks. Load and validate bounded immutable input snapshot first, evaluate in memory, write blob, then insert the verified reference. Use existing three-second busy-timeout behavior and handle conflicts safely.
- Existing Store uses immediate write transactions, `synchronous=FULL`, delete journal mode, and immutable triggers for event/verification/blob update (`aether-store/src/lib.rs:101-136`, `001_initial.sql:143-154`). Those guarantees do not cover the model registry or prove cryptographic provenance.

### Errors

Map expected failures deliberately: `400` invalid request/schema, `404` missing artifact/report, `409` digest/version integrity conflict, `503` internal/storage failure, and a bounded `429`/`503` for admission exhaustion. Invalid research evidence is a report check outcome; it is not always an HTTP request error. Avoid mapping every evaluator error to `400` as the current APO-10 handler does.

## 8. Human review: viable now vs gated

### What current records can establish

- The configured bearer token establishes that a request supplied the local shared secret. It does not identify who possesses it.
- `reviewer: "local_operator"` in approval receipts establishes that the local record was labeled as a local operator action, not which human performed it (`models/workflows.rs:111-126,822-855`).
- Content hashes can detect accidental byte changes if a trusted digest is retained. Local SQLite/artifact files can be replaced together by a person with filesystem access; hashes are not signatures.
- Current verified campaign records establish only the frozen rule/evaluator outcome after required evidence checks (`aether-store/src/evidence.rs:150-204`). They do not establish the independence of a literature reviewer.

### Future policy, subject to owner approval

- Authenticate a stable reviewer principal from a trusted identity context; never accept a principal string from the client body. Define roles (operator, reviewer, adjudicator) and prohibit self-review where independence is claimed.
- Require a per-review conflict-of-interest disclosure (e.g. none declared, financial, authorship, institutional, supervisory, other) and optional recusal. Treat disclosure as self-attested, not externally verified.
- Pin exact report digest, rubric ID/version/digest, dimension, verdict, evidence passage digests, bounded rationale, reviewer principal, and event time. Preserve disagreements as separate records.
- Corrections and appeals append a new event with `supersedes`/`appeals` reference and reason; no edit/delete. Resolution requires an explicit authorized resolution event and leaves original votes visible.
- A hash chain and SQLite update/delete triggers can make local history tamper-evident, not tamper-proof. Do not call it signed or independently verified unless a separately approved signature/custody scheme is implemented and tested.
- Human review never grants execution permission, changes the trusted verification store, or upgrades `EvidenceReference` to verified. One local operator cannot be counted as independent review.

## 9. API convention investigation

Observed source: `crates/aether-api/src/http.rs:68-132` registers protected feature routes under `/v1` (`/v1/workflows`, `/v1/workflow-benchmarks`, `/v1/knowledge`, etc.). The existing content-addressed pattern is `/v1/workflow-benchmarks/{sha256}` (`:115-120`). All protected routes receive the body limit and bearer middleware (`:131-132`).

The shared ledger’s `CTR-EVAL-02` draft lists `POST /api/v1/eval/quality` and `GET /api/v1/eval/reports/:id` (`docs/agent-handoff/AGENT_SHARED.md:136`). That path is not registered by the current router. The earlier Codex proposal recommends `/v1/eval/...`, which follows current source but conflicts with the shared draft.

**Recommendation for reconciliation, not route approval:** compare with Antigravity’s independent finding. If no API gateway or uninspected route rewriter owns `/api`, align to the source convention `/v1`; use `{sha256}` for content-addressed report inspection. Do not register both prefixes as aliases merely to mask the conflict. Do not finalize the prefix until the two reviews and project owner reconcile it.

Separate API capabilities:

1. `POST` evaluation request: runs deterministic local structural checks and writes one report; no human judgment, no independent validation, no external calls.
2. `GET` report list/detail: read-only, authenticated, bounded pagination, digest revalidation.
3. Human review writes: disabled pending identity/policy approval.
4. Independent scientific validation: no endpoint in this milestone.

Use typed errors instead of passing raw evaluator errors into `Error::Input`. Keep all report inspection routes read-only.

## 10. Smallest useful implementation order

1. **Contract and evidence records (no routes):** after owner approves the shared schema, add closed typed request/reference/report models, exact artifact/source/passage/provenance identity, fixed all-eight-dimension output shape, and versioned rule IDs. Do not add human-write models to public API yet.
2. **Pure deterministic validator:** add fixture-based checks for source/revision/passage integrity, provenance completeness, and reproducibility manifest/receipt integrity. These three make a useful narrow vertical slice over existing graph and workflow artifacts. For the first release, the other five dimensions must appear as explicit `unavailable: rule_not_implemented_v1` rather than fabricated passes. No model calls or execution.
3. **Expand structural rules:** add counterevidence, hypothesis, design, method, and uncertainty field checks only once their exact source schemas/rules are pinned. Keep them structural; if a schema/source is absent or truncated, emit unavailable/inconclusive.
4. **Content-addressed report generation:** serialize with a named canonicalization version; hash required pins/results; fresh-process known-answer replay; persist blob then immutable registry reference with crash tests.
5. **API/CLI:** implement one structural evaluation trigger plus authenticated read-only list/detail routes after route reconciliation. Do not implement a reviewer write route. Keep status classes consistent with the approved contract.
6. **Human review:** later milestone only after principal/role, conflict, disagreement, appeal, retention, append-only, and local-vs-external identity policy are approved.
7. **Independent scientific validation:** separate future protocol and independent custody. It remains `not_performed`; no APO-11 structural report can change it.

Do not create a new database table for structural reports unless the existing artifact/registry path cannot meet bounded persistence needs. If human review is approved later, use a separate append-only event table rather than the model registry UPSERT helper.

## 11. Adversarial test plan

- **Dimensions:** each of the eight dimensions has required-field pass/fail, unsupported schema, absent artifact, malformed reference, truncation, and budget-exhaustion cases. Assert all results are `structural_completeness`; no aggregate score exists; unsupported five-dimension v1 rules remain unavailable.
- **Evidence identity:** same source/different passage; same passage/different source version; stale source hash; out-of-bounds and non-UTF-8-boundary offsets; extraction representation mismatch; duplicate source groups; unresolved pointer; structurally valid evidence remains unverified.
- **Authority:** request attempts to set `verified`, reviewer principal, independent validation, approval, or model judgment are rejected/ignored by typed schema; only trusted campaign verification can provide its existing protocol-scoped state.
- **Determinism:** golden canonical bytes/digest vectors for each schema; integer bounds and decimal representation; NFC/NFD-distinct source bytes; no timestamps/rowids/PIDs; stable ordered arrays/maps; mutation changes digest; two fresh processes match; different executable/config pin does not falsely compare as identical.
- **Resource/security:** oversized body/artifact/bundle/report, deep JSON, duplicate refs, integer overflow, excessive graph edges, truncated graph, concurrency saturation, cancellation before/after snapshot, unauthorized token, no raw source text/error detail.
- **Persistence:** injected crash after blob write and before registry insert leaves no visible report; registry insert failure leaves orphan blob only; duplicate digest verifies equality; corrupted/missing blob returns integrity/not-found; concurrent equal inserts are idempotent; no UPSERT replacement.
- **Review (future, tests only until approved):** anonymous reviewer rejected, caller-supplied principal ignored, conflicting votes preserved, conflict-of-interest declaration retained, correction/appeal appends, no update/delete, no identity/signature/independence claim from local operator label.
- **API:** verify route auth, current body cap, list truncation, digest path, status mapping, and that report GET cannot mutate state.

These tests validate software contracts, not scientific validity, source authenticity, reviewer independence, or quality of findings.

## 12. Decisions requiring owner approval

1. Which API prefix contract wins after reconciliation with Antigravity: current source `/v1` or shared draft `/api/v1`?
2. Approve the first slice as structural-only, with the five not-yet-implemented dimensions explicit unavailable, or require all eight structural schemas before any release?
3. Approve the source/passage/provenance reference types and canonicalization/hash version before public API exposure.
4. Confirm that human review writes stay disabled until a reviewer principal/role model exists. Current local bearer and `local_operator` label do not meet an independence claim.
5. Decide whether any future local operator review should be labeled only “local attestation” and what conflict-of-interest/appeal retention policy applies.
6. Confirm that independent scientific validation remains a separately custodied future process and that APO-11 cannot set its status.

## 13. Source references and limits

- API routes/auth/errors/body cap: `crates/aether-api/src/http.rs:37-64,68-132,157-189`.
- APO-10 serialization, replay, and persistence: `crates/aether-api/src/models/workflow_evaluation.rs:299-307,638-704,707-792`.
- Registry table/upsert: `crates/aether-api/src/models.rs:247-276`.
- Artifact lookup/write and registry DB location: `crates/aether-api/src/lab.rs:215-239,626-654`; `crates/aether-store/src/execution.rs:303-317`.
- Graph source/status/revision and snapshot: `crates/aether-api/src/literature/knowledge.rs:85-165,250-321,524-652`; `crates/aether-api/src/literature/knowledge.sql:1-40`.
- Campaign evidence/verdict and immutable triggers: `crates/aether-store/src/model.rs:36-48,215-263`; `crates/aether-store/src/evidence.rs:62-100,150-204`; `crates/aether-store/src/001_initial.sql:89-120,130-154`.
- Local workflow approval label: `crates/aether-api/src/models/workflows.rs:111-126,822-855`.
- Shared route draft: `docs/agent-handoff/AGENT_SHARED.md:136` (read only; not modified).

No tests, builds, benchmarks, model calls, external API requests, source changes, migrations, frontend changes, shared-ledger edits, held-out data, or frozen artifact modifications were made for this review.
