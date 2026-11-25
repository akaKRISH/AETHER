# 06 | Detailed system design (LLD)

## Modules and contracts
| Module | Core interface | Responsibility |
|---|---|---|
| domain | Protocol, Run, Claim, Evidence | Immutable typed objects and transitions. |
| protocol | freeze(config), approve(hash) | Validate task and approval. |
| scheduler | choose(state, remaining) -> Action | Rank eligible actions; return STOP if none. |
| budget | reserve, settle, release | Atomic vector caps, monotonic usage. |
| runner | submit(spec), cancel(id), recover() | Leases and isolated execution. |
| evaluator | score(predictions, task), replay(spec) | Trusted metrics and independent attempts. |
| ledger | append(event), project_state() | Event persistence and audit. |
| report | export(campaign_id) | Render from ledger, not free text. |

## State transitions
Campaign: DRAFT -> APPROVED -> RUNNING -> COMPLETED, FAILED or CANCELLED. RUNNING -> PAUSED -> RUNNING requires valid approval and remaining budget; crash recovery uses persisted state. Run: QUEUED -> LEASED -> RUNNING -> SUCCEEDED / FAILED / CANCELLED / EXPIRED. Terminal records immutable; retries use new IDs. Claim: PROPOSED -> SUPPORTED -> VERIFIED, REJECTED or INCONCLUSIVE. Invalidation appends a REVOKED event and creates a new current projection; never erase historical verdicts. A claim is VERIFIED only if all required edges resolve, evaluator passed and prescribed replay/challenge passed. Missing evidence is INCONCLUSIVE, not success.

## ECAR policy v0: explicit, inspectable heuristic
Action kinds: EXPLORE, REPLAY, CHALLENGE, STOP. Each action has estimated resource vector, prerequisites, affected claims, estimated utility and evidence uncertainty. Filter actions that violate policy, dependencies, or remaining budgets; reserve before dispatch. Rank remaining actions by `(expected_gain + lambda * uncertainty_reduction + mu * claim_importance) / normalized_estimated_cost`. Normalize cost as maximum fraction of positive remaining caps consumed (with positive epsilon); a zero remaining cap permits only zero use of that resource. Use a fixed deterministic tie-breaker and logged coefficients. Start with lambda=mu=1 as pilot defaults; freeze after development-set calibration. Score estimates initially from task heuristics and observed run history; no claim of calibrated probability. Require a reserved replay allowance (initially 25% of time and token budgets) before exploration can consume resources. Stop on approved objective, exhausted caps, cancellation, no feasible action or max attempts. Independent verifier, not policy, determines validity.

## Concurrency and correctness
Single worker and SQLite transactions in R0. Reserve and create job in one transaction; lease includes attempt ID, expiry and fencing token. On result commit check current lease and unique attempt; stale workers cannot publish. Reservation tracks estimated upper bound; actual usage settled once; reconcile crash leases conservatively. Local event journal and result write use transaction; artifact blobs are staged then hashed and moved atomically before DB references are committed. Orphan blobs collected after retention window.

## Verification contract example
Claim: candidate mean accuracy >= baseline mean accuracy - 0.01 and median latency <= 0.90 * baseline on frozen hardware and evaluation set. Evidence: baseline manifest, candidate manifest, metric digest, replay manifest, paired timing samples, evaluator version. If labels, hardware, tolerance or code digest mismatch, return INCONCLUSIVE. If valid evaluation disproves the predicate, return REJECTED. A report may show unverified hypotheses but clearly marks their status.
