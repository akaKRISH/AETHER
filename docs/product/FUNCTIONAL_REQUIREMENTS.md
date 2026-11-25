# Functional requirements

IDs are carried forward from [v0.2 PRD](../archive/v0.2/AETHER/03_PRD.md). Release tags denote the complete capability. The Phase 1 fixture has its own acceptance IDs.

| ID | Release | Required behavior | Acceptance |
|---|---|---|---|
| PRD-F01 | R0 | Create local project and select a task | Valid config is persisted and hashed. Invalid task or budget produces a typed rejection with no work queued. |
| PRD-F02 | R0 | Freeze and approve protocol | Worker launch needs matching approval for exact digest. Editing frozen inputs changes the protocol revision and voids that approval. |
| PRD-F03 | R0 | Run baseline | Each terminal attempt records status, logs or a typed reason, manifest and metric record when valid. |
| PRD-F04 | R0 | Inspect evidence | Every exported quantitative claim resolves to immutable source artifact and metric. Missing references block verified export. |
| PRD-F05 | R0 | Cancel and recover | Supported supervisor stops the full process tree within budgeted limits and recovers only eligible work. |
| PRD-F06 | R1 | Generate candidate hypotheses | Accept typed proposal with rationale, metric and cost estimate. Deny unknown task and malformed fields before scheduling. |
| PRD-F07 | R1 | Allocate approved resources | Reservation enforces all caps and the verification reserve. Expired budget prevents new work. |
| PRD-F08 | R1 | Require independent verification | Fresh execution and frozen evaluator satisfy protocol. Failed or missing replay prevents VERIFIED. |
| PRD-F09 | R1 | Export evidence bundle | Bundle includes protocol, code digests, all attempts, evidence, verdict history and caveats with checksums. |
| PRD-F10 | R2 | Use web workspace | Keyboard users can view runs, compare metrics, inspect evidence/budget and cancel authorized campaigns. |
| PRD-F11 | R2 | Opt into remote GPU | Authorize job, validate inputs/outputs and scope credentials. Unapproved, stale or mismatched jobs fail closed. |
| PRD-F12 | R3 | Extend through task SDK | Reviewed task pack passes contract and denial tests without changes to core. |

See [test mapping](../testing/REQUIREMENTS_TRACEABILITY.md).
