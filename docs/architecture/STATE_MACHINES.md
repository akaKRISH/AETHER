# Campaign, run and claim state machines

Validate each transition against its current persisted state. Retry receives a new attempt ID. Scientific reevaluation after a terminal verdict receives a successor revision. Preserve both in audit history.

| Object | Legal transitions |
|---|---|
| Campaign | DRAFT → APPROVED → RUNNING → COMPLETED, FAILED or CANCELLED. RUNNING → PAUSED → RUNNING rechecks approval and feasible budget. PAUSED → CANCELLED permits cleanup. |
| Attempt | QUEUED → LEASED → RUNNING → SUCCEEDED, FAILED, CANCELLED or EXPIRED. QUEUED may cancel. LEASED may fail, expire or cancel before its executable starts. |
| Claim | PROPOSED → SUPPORTED after trusted preliminary evidence. PROPOSED may reach REJECTED after valid contrary evidence or INCONCLUSIVE when required evidence is unavailable. SUPPORTED → VERIFIED, REJECTED or INCONCLUSIVE. VERIFIED → REVOKED by appended invalidation event. |

VERIFIED requires trusted predicate pass, all mandatory evidence, eligible dependencies and required independent replay/challenge. The [source prose](../archive/v0.2/AETHER/06_LLD.md) depicts SUPPORTED before all outcomes. Its [state diagram](../archive/v0.2/AETHER/diagrams/claim_states.mmd) permits early negative outcomes. The proposal retains early negative paths with rule-based reasons. Every path to VERIFIED includes SUPPORTED.

Cancellation closes admission immediately. Terminal cancellation follows reconciliation and settlement. Lease fencing applies before any result commit. See the [state diagrams](../diagrams/README.md).
