# 04 | Technical requirements / SRS

## Functional invariants
Every campaign references one immutable protocol and policy version. Every run uses a unique attempt ID, pinned task version, code digest, input hashes and seed. Only verifier code may transition a claim to VERIFIED. A result is not scientific evidence until metrics pass schema and provenance checks. The held-out evaluator is inaccessible to candidate code; candidates cannot write the ledger, test labels or verifier. All model and user commands are treated as untrusted input.

| ID | Proposed measurable requirement | Verification |
|---|---|---|
| TRD-N01 | Local CPU path runs without GPU; one worker default. | Integration test on CPU. |
| TRD-N02 | Atomic budget reservations; no new action over caps. | Concurrent reservation tests and crash test. |
| TRD-N03 | Run cancellation initiated within 2 seconds; CPU process killed within 10 seconds on supported local runtime. | Timed process-tree test; GPU teardown separate. |
| TRD-N04 | Artifact hashes checked on ingest and replay; tampering blocks export as verified. | Change one byte and rerun validation. |
| TRD-N05 | API p95 <= 500 ms for local metadata read under 5 clients, excluding model/worker work. | Recorded load test R2. |
| TRD-N06 | No candidate network, secrets, host mounts or Docker socket. | Denial tests in controlled environment. |
| TRD-N07 | Recovery is at-least-once with idempotent commit; no duplicate committed result. | Kill worker after artifact write before commit. |
| TRD-N08 | Backup restore target RPO 24h / RTO 2h for local metadata and artifacts. | Restore drill. |
| TRD-N09 | Every run has structured start/end events, budget usage and error class. | Schema/coverage tests. |
| TRD-N10 | UI keyboard navigation and WCAG 2.2 AA target for R2. | Keyboard/manual and automated accessibility checks. |

## Initial resource envelope
Assumed developer laptop: i5-13420H, RTX 3050 6 GB; RAM and storage to verify. CPU task: one worker, 2 CPU threads, 2 GiB worker memory, 5-minute execution cap, 200 MiB output quota. Campaign default 30 min wall-clock, 20 candidate attempts, 60,000 model tokens and zero paid credits; any paid spend requires explicit approval. Use CPU first; GPU only after runtime and memory tests. Optional free cloud credits are not guaranteed. R0 has no model API dependency when using scripted candidates. Exact limits calibrated by pilot, then frozen for each experiment.

## Failure semantics
Reject invalid configuration before allocation. Categorize timeout, OOM, policy violation, evaluation failure, artifact error, cancellation and infrastructure interruption. Only transient infrastructure failures receive one automatic retry with a new attempt ID and accounted cost; invalid candidates are not silently replaced. Any budget overrun from a provider's delayed usage report is recorded, scheduling halted, and treated as protocol deviation.
