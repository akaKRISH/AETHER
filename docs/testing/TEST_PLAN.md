# Test plan

Original IDs follow [v0.2 section 15](../archive/v0.2/AETHER/15_TESTING_QA.md). All application outcomes below are proposed checks.

| Test | Gate | Setup and assertion |
|---|---|---|
| T01 | R0/R1 | Reject missing task, invalid budget or malformed proposal before allocation. |
| T02 | R0 | Approve one protocol hash, change frozen bytes; deny launch. |
| T03 | R0 | Run baseline twice with pinned inputs; validate manifest completeness, metrics and tolerances. |
| T04 | R0/R1 | Competing reservations and duplicate settlement preserve caps and a single terminal effect. |
| T05 | M1/R0 | Modify fixture manifest or stored blob; fail integrity and block verified report. |
| T06 | R0 | Worker claims a score inconsistent with predictions; authoritative metric uses trusted evaluator. |
| T07 | R1 | Missing replay/evidence or contradictory outcome yields protocol-defined INCONCLUSIVE or REJECTED. |
| T08 | R0 | Interrupt after artifact staging before database commit; recover without duplicate result. |
| T09 | R0 | Cancel reviewed fixture with child process; start ≤2 s and terminate process tree ≤10 s on supported local CPU runner. |
| T10 | R0 | Controlled denial fixtures exercise network, credentials, host mounts and path boundaries. |
| T11 | R0 | Reviewed limit fixtures exceed resource quotas; terminate, bound output and reconcile cost. |
| T12 | R0 | Restore clean location; assert ledger/blob closure, RPO ≤24 h and RTO ≤2 h in drill. |
| T13 | R2 | Disconnect and reconnect SSE at sequence cursor; retain coherent deduplicated state. |
| T14 | R2 | Complete setup, inspect, compare and cancel using keyboard; check WCAG 2.2 AA criteria. |
| T15 | R1 | Run A–D comparison arms in dev harness; identical task/evaluator/caps, complete outcome ledger. |
| T16 | R1 | Reject dependency cycle; upstream revocation withdraws affected verified projection. |
| T17 | M1 | Fresh build and fixture in approved environment without Python/GPU/LLM. |
| T18 | R0 | Reject unknown worker schema, malformed/oversized result, mismatched ID or denial. |
| T19 | R0 | Worker cannot access/write authoritative data or hidden labels. |
| T20 | M1/R0 | Lint first-party core for unsafe, audit the actual lockfile at release. |
| T21 | R0 | On supported Linux, crash and cancel in lifecycle tests; enforce bounded cleanup. |
| T22 | R0 | Reviewed Python predictor emits fixtures; Rust evaluator matches independently calculated metrics. |
| T23 | M1 | Exact counts and invalid values, empty input and unequal lengths; init/re-run/replay/report in fresh invocations. |
| T24 | R2 | Five local metadata clients with representative data; p95 ≤500 ms, explicitly exclude long worker/model steps. |
| T25 | R2 | Invalid authorization, stale lease or digest mismatch remote result never commits; no worker secrets. |
| T26 | R3 | Reviewed pack passes conformance and denial tests with no patch to core. |
| T27 | R1 | Approved bundle includes all attempts, failure/caveat records, pinned versions and valid checksums. |
| T28 | R0 | Retrieved text or candidate proposals request a denied operation, or carry an unknown version. No side effects and a recorded denial. |
| T29 | R0 | Build/release review verifies dependency, image, and license pins and excludes secrets from exports. |

M1 behavior is further specified in the [native milestone](../roadmap/MILESTONE_01_NATIVE_FIXTURE.md). Record environment, exact commands, expected/actual state and required artifacts for each run.
