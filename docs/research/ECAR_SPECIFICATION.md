# Evidence-Constrained Adaptive Research

**Status** — Proposed v0 policy for later development and pilot calibration. The first native classification fixture establishes metric and evidence semantics. Scheduler evaluation originates in the development pilot.

## State and authority

The Rust controller receives the campaign's frozen protocol and policy digest, approved objective, scored action candidates, typed claim graph and authoritative budget snapshot. A claim may require an artifact, an evaluator result, another claim or independent replay or challenge evidence. Dependency edges form a DAG. A worker can submit bounded results and provenance. Only the trusted evaluator can decide whether a claim meets frozen verification rules.

| Action | Purpose | Admission examples |
| --- | --- | --- |
| EXPLORE | Test a new hypothesis or candidate | Registered task, pinned inputs and a protected verification reserve |
| REPLAY | Produce a fresh attempt at an existing claim | Frozen inputs, replay policy, new attempt ID and clean work directory |
| CHALLENGE | Try a preregistered disconfirming test | An applicable rule, approved challenge scope and an evaluator |
| STOP | End scheduling with an explicit reason | Objective met, exhausted caps, cancellation, no feasible action or attempt limit |

## Decision sequence

1. Honor cancellation and verify whether the frozen objective has already been satisfied. Enumerate actions with stable IDs from approved task and claim records.
2. Reject missing prerequisites, incompatible approval, invalid estimates, exhausted attempt caps and actions whose upper bounds exceed remaining spendable resources. If exploration would violate the protected reserve, omit it. An unavailable action logs a denial reason.
3. Rank eligible actions by estimated gain, uncertainty reduction and affected claim importance divided by normalized estimated cost. Use the deterministic comparison in [Mathematical foundations](MATHEMATICAL_FOUNDATIONS.md). Start pilot weights at lambda = mu = 1. Freeze weights, estimate functions, and tie-breaking before the held-out study.
4. The engine atomically reserves the estimate and creates the attempt. If a concurrent or recovered update makes it unaffordable, reload current state before choosing again. The authoritative controller records policy version, eligible scores, selected action, reason, budget snapshot and resulting attempt identifier. Worker retries consume a new attempt and budget.
5. The verifier records supported evidence, disconfirming evidence or missing evidence against the frozen protocol. The scheduler consumes updated state on the next decision. An invalidation event preserves prior verdict history and can change the current state of dependent claims.

Reserve 25% of initial wall-time and model-token caps for independent verification as the v0.2 pilot default. Proposed interpretation for review: round positive integer floors up, debit verified replay/challenge commitments against the floor, and release unused reservations when settled or cancelled. Preserve the unconsumed floor for verification until terminal STOP. A zero token cap protects zero tokens and admits actions with zero token estimates. Freeze rounding and release semantics before comparisons.

Replicating one flawed evaluator can repeat its error. Match inputs and protocol while separating proposer, fresh attempt and evaluator authority. When warranted, add a frozen hidden challenge or independent evaluator. Scheduler urgency never relaxes a verdict requirement.
