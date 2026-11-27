# APO-11 Structural Evaluation Backend

**State:** Implemented local structural-only slice; canonical wire contract is [`contracts/apo11/quality-evaluation.schema.json`](../../contracts/apo11/quality-evaluation.schema.json), with benchmark definition and golden fixtures beside it.

## Purpose and limits

This backend checks whether selected research artifacts contain required structural fields and internally consistent references or arithmetic. It does not judge literature relevance or quality, claim entailment, scientific novelty, calibration, or truth. A `pass` is a structural result only. Source identity fields are preserved input claims; the evaluator does not authenticate those identities or verify evidence. Human-review write APIs are absent. `independent_scientific_validation` is always `not_performed`, and these results must never be used as generator optimization or selection objectives.

Evaluation reports are stored only after the evaluator completes. A completed report may contain `fail` or `unavailable` dimensions. An invalid request, missing snapshot, storage failure, or digest mismatch returns an API error and creates no report. Evidence references remain unverified unless another trusted process establishes verification.

## Wire contract and endpoints

All routes use the existing bearer-authenticated loopback `/v1` router and the existing JSON error envelope `{ "code": string, "error": string }`. The schema is closed (`additionalProperties: false` for request and response records) and versions every document.

| Method and route | Behavior |
|---|---|
| `GET /v1/eval/benchmarks` | List the fixed benchmark ID/version, its raw benchmark digest, structural scope, and eight dimension IDs. |
| `GET /v1/eval/benchmarks/{sha256}` | Return the benchmark only when the lowercase SHA-256 matches the embedded benchmark bytes. |
| `POST /v1/eval/snapshots` | Resolve a bounded graph export and typed subject references, create a rights-safe immutable input projection, and return its digest receipt. New snapshot: `201`; identical existing snapshot: `200`. |
| `POST /v1/eval/runs` | Validate the requested benchmark and snapshot, evaluate the fixed rules, persist a completed report, and return its report digest and stable sequence. New report: `201`; identical replay: `200`. |
| `GET /v1/eval/reports?limit=N&before_sequence=S` | List newest-first report summaries. `limit` defaults to 32 and must be 1–32. The optional cursor is exclusive and must be positive. Follow `next_before_sequence` while `truncated` is true. |
| `GET /v1/eval/reports/{sha256}` | Verify and return the stored report envelope identified by its raw report digest. |
| `GET /v1/artifacts/{sha256}` | Existing endpoint returns exact raw artifact bytes; clients can independently hash snapshot and report payloads. |

The two-step flow lets clients inspect and retain an immutable snapshot receipt before running an evaluation. Minimal request examples:

```json
{"schema":"aether.quality-snapshot-request/1","graph_root_id":"2222222222222222222222222222222222222222222222222222222222222222","graph_depth":2,"subjects":[],"generate_hypotheses":false,"generate_experiment_designs":false}
```

```json
{"schema":"aether.quality-run-request/1","benchmark_sha256":"1111111111111111111111111111111111111111111111111111111111111111","snapshot_sha256":"3333333333333333333333333333333333333333333333333333333333333333"}
```

These digests are schema-valid fixture values only; use the actual graph root, benchmark catalog and snapshot receipt digests for a live request.

`SubjectRef` carries both the subject kind and a typed `ArtifactRef` (`hypothesis_report` or `experiment_design_report`) plus a JSON Pointer. The service rejects duplicate references, kind mismatches, malformed digests, non-pointer paths, and excessive request counts. The exact required and returned JSON fields are defined by the schema and examples in [`contracts/apo11/fixtures/`](../../contracts/apo11/fixtures/).

### Errors

| HTTP | `code` | Meaning |
|---|---|---|
| 400 | `invalid_request` | Malformed JSON, unknown fields, invalid digest/typed reference, invalid query, or documented count limit exceeded. |
| 401 | Existing auth error | Missing or invalid bearer token. |
| 404 | `not_found` | Unknown benchmark, report, snapshot, or artifact. |
| 409 | `state_conflict` | Source graph changed while its snapshot was captured. |
| 409 | `integrity_failed` | Stored artifact or report fails raw-byte digest or schema checks. |
| 413 | `request_too_large` | Actual Axum JSON body-limit rejection; uses the existing `{code,error}` envelope and 8 KiB limit. |
| 503 | `operation_failed` | Transient or unexpected storage/operation error; internal details are not returned. |

No failed or partially evaluated result is stored as a successful report. Snapshot capture checks the graph digest before and after projection; a changed graph fails with `409 state_conflict`.

## Fixed structural dimensions

Each report includes exactly one result for every fixed dimension. Dimension result is a deterministic roll-up of its checks: any `fail` gives `fail`; otherwise all `pass` gives `pass`; any remaining mix containing `unavailable` gives `unavailable`. Empty checks are invalid. Execution outcome is separate: persisted reports have `execution_outcome: "completed"`, including reports with failed structural checks.

| Dimension | Current structural rule | Empty or unavailable behavior | Cannot establish |
|---|---|---|---|
| `literature_scope_inventory_integrity` | At least one paper node in a non-truncated graph projection. | Empty required inventory fails; truncated inventory is unavailable. | Coverage, relevance, source quality, or truth. |
| `claim_source_locator_integrity` | Claims link through typed `supports` or `contradicts` edges to evidence with structurally valid locators. Passage offsets and hashes are checked for shape and ordering. | Empty required claims fail; missing/legacy locators are unavailable; malformed explicit locators fail. | Whether a cited passage entails or supports a claim. |
| `evidence_provenance_field_integrity` | Evidence origin fields are structurally valid; explicit source-identity and epistemic-status fields are present. | Empty optional evidence is unavailable; malformed provenance fails; missing status is unavailable. | Authenticity or correctness of asserted provenance/status. |
| `counterevidence_edge_integrity` | Contradiction edges preserve their directed relation and point to existing, distinct nodes. | No contradiction edge is unavailable; missing/self endpoints fail. | Whether counterevidence is scientifically relevant or actually contradictory. |
| `hypothesis_structure_completeness` | Nonempty proposition, support and alternatives, assumptions, limitations, falsification condition, metrics, and resource estimate. | Empty required hypothesis set fails; missing/empty fields fail. | Novelty, plausibility, test success, or truth. |
| `experiment_design_arithmetic_integrity` | Required variables, controls, baseline, confounder analysis, data separation, sample-size basis, statistical plan, metrics, falsification/termination and checked budget sums are present. | Empty required design set fails; missing fields, overflow, or inconsistent sums fail. | Statistical power, feasibility, fairness, or scientific validity. |
| `uncertainty_field_wellformedness` | Hypothesis uncertainty grade is within the declared 0–4 range and epistemic status is explicit. | Empty required hypotheses fail; missing grade/status is unavailable; out-of-range grade fails. Explicit `unknown` is distinct from a missing status. | Calibration or whether uncertainty is justified. |
| `artifact_snapshot_replay_integrity` | Snapshot and graph digests have been verified; graph truncation and source replay state are explicit. | Digest mismatch fails; truncated graph or unavailable source replay is unavailable. | Replay of source material not retained under known storage rights. |

The benchmark pins a versioned `rule_id` for each dimension. `StructuralCheck` contains that ID, outcome, and stable reason codes; clients resolve descriptive text from the benchmark instead of duplicating display strings in each check.

## Snapshot, provenance, and persistence

Snapshot capture stores a typed structural projection, not source prose. It retains graph/source digests, byte offsets, typed source locators, origin, explicit source-identity status, and epistemic status as available. Dynamic APO-03 hypothesis and APO-04 design reports can be resolved by immutable artifact digest and JSON Pointer; the resulting projected facts are stored before evaluation. Generated statement text is removed from the projection. Source text is never copied or sent to another service. Therefore current source replay is `unavailable`, including for rights-unknown literature.

Digest fields identify different bytes:

- Source/artifact SHA-256 is the raw immutable source artifact bytes.
- Graph SHA-256 is the graph export payload digest returned by the existing graph catalog.
- Snapshot SHA-256 is raw serialized snapshot-document bytes.
- Report SHA-256 is raw serialized `Report` bytes, not the containing envelope or API JSON reserialization.
- Benchmark SHA-256 is raw bytes of `contracts/apo11/benchmark.json`.
- Evaluator version identifies the code contract. Optional executable SHA-256 is self-reported; build attestation is `not_provided`, so this is not a complete evaluator build identity.

The existing Store holds immutable content-addressed snapshot and report blobs. The existing Registry holds a narrow additive report-sequence index and local model-registry entry; insertion is idempotent and ordered by an explicit AUTOINCREMENT sequence. Listing uses the exclusive sequence cursor, so ordering survives `VACUUM` and restart. Blob storage and registry indexing are separate SQLite databases and are **not** one atomic transaction; a failed index write can leave an orphaned immutable blob. No report is listed unless its registry entry and referenced bytes pass integrity validation.

Serialized report bytes use the current `serde_json::to_vec` struct field order and are hashed exactly as stored. This is a same-implementation/same-version byte contract, not a claim of universal canonical JSON across languages or builds. Timestamps are not included. Any future incompatible field or serialization change must use a new schema/evaluator version; older immutable report bytes remain readable only under their schema contract and are never silently rewritten.

## Limits and security

The benchmark and implementation currently agree on these hard ceilings: request body 8 KiB; graph depth 3, 64 nodes and 192 edges; 8 subject references; 2 MiB per referenced artifact; 4 MiB snapshot; 1 MiB report; 256 KiB report-list response; page size 1–32. These are admission ceilings, not caller-selected budgets. Parsing is strict and bounded; report generation runs synchronously on the existing blocking path. The endpoints inherit the existing bearer authentication and loopback-only server behavior.

No human-review writes, model calls, external-provider access, arbitrary execution, training, or held-out data are involved. Local authenticated access identifies possession of the configured local bearer token; it does not establish reviewer identity or external proof.

## Contract validation and tests

With the existing `requirements-validation.txt` environment installed, run:

```powershell
python scripts/validate_apo11_contract.py
cargo test -p aether-engine quality::tests --lib
cargo test -p aether-api --bin aether-lab models::quality::tests
cargo test -p aether-api --test platform apo11_structural_evaluation_is_authenticated_bounded_replayable_and_content_addressed
```

The validator checks Draft 2020-12 schema validity, the benchmark definition, and every JSON fixture. Rust tests cover all eight dimensions, vacuous empty sets, unknown versus missing status, counterevidence polarity, legacy locators, malformed typed references/enums/fields, benchmark pins, report roll-up and exact report bytes. The API regression exercises bearer auth, snapshot/run idempotency, exact raw report digest, report pagination before/after `VACUUM`, fresh-server replay, invalid JSON, body-limit `413`, and corrupted blob rejection. `scripts/validate_docs.py` runs the project documentation checks. Required validation outputs are recorded in `RESUME.md` and `docs/testing/REQUIREMENTS_TRACEABILITY.md`.

Frontend integration is read-only: fetch the benchmark catalog/detail, create a snapshot, inspect or retain its receipt, then submit an operator-triggered run and render the eight dimension outcomes. Use the benchmark's versioned rule metadata for labels/help text, display `execution_outcome` independently from structural outcomes, show `unavailable` without converting it to pass/fail, and never label these results as evidence verification or scientific quality.
