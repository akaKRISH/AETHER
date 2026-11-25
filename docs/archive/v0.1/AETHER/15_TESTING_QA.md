# 15 | Test strategy and quality plan

| Test ID | Scenario | Expected outcome |
|---|---|---|
| T01 | Invalid protocol, budget or task | Reject before worker. |
| T02 | Approve config then modify hash | Execution denied. |
| T03 | Baseline on pinned inputs twice | Metrics within declared tolerance; manifests complete. |
| T04 | Concurrent reservations and double settle | Caps never oversubscribed; settle once. |
| T05 | Tamper with blob / manifest | Hash validation fails; verified export blocked. |
| T06 | Candidate returns false metric | Trusted evaluator ignores candidate verdict. |
| T07 | Replay fails or evidence missing | REJECTED or INCONCLUSIVE, not VERIFIED. |
| T08 | Kill worker before commit | Reconcile lease, no duplicate committed run. |
| T09 | Cancel task spawning child processes | Process tree stops within local target. |
| T10 | Denied network, secret, path traversal | Policy denial; no host data exposed. |
| T11 | Exhaust CPU/memory/disk limits | Bounded failure; accounting and cleanup. |
| T12 | Restore from backup | Project, runs, events and artifacts restored; digests match. |
| T13 | SSE disconnect and duplicate event | Resume sequence; no duplicate UI state. |
| T14 | Keyboard-only navigation | All R2 controls operable, focus visible. |
| T15 | All four research configurations | Same task/evaluator/caps; complete evidence and failure logs. |
| T16 | Claim dependency cycle or revoked upstream claim | Cycle rejected; dependent claim no longer verified. |

## Test pyramid and release gates
Unit tests: transitions, graph, budget, ranking, hashes, schema and metrics. Property tests: budget conservation, graph acyclicity, idempotence. Integration: controller/runner/evaluator/artifacts, crash recovery, security fixtures. End-to-end: CLI task through export. R2 adds API authorization, UI accessibility, load and remote worker tests. Aim for >= 80% branch coverage in policy/budget/ledger core, but coverage alone is not evidence of correctness. All critical invariants and denial tests must pass; no unresolved critical/high vulnerabilities in shipped paths. Record exact environment, seed and test command in CI results. Test specifications only; no tests have been executed against AETHER because implementation has not begun.
