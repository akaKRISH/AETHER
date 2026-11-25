# Use cases

| Workflow | Preconditions | Success path | Failure behavior |
|---|---|---|---|
| Learn the fixture | Initialized writable local project | Run pinned synthetic inputs, record exact counts, replay, read report | Reject malformed inputs or corrupt manifest with typed errors |
| Run approved baseline | Reviewed pack, pinned protocol, approval and supported runner | Reserve resources, execute, score in trusted context, record all output | Stop or cancel, retain failure evidence and settle budget |
| Examine a claim | Campaign has trusted evidence | Traverse metric, blob, replay and dependent claims | Show missing, contradictory or revoked evidence |
| Reproduce study | Access to permitted frozen artifacts and environment | Execute same evaluator, compare to registered tolerance and publish deviations with authorization | Record incompatible environment or missing artifacts |

See the [first milestone](../roadmap/MILESTONE_01_NATIVE_FIXTURE.md) and [reproducibility protocol](../research/REPRODUCIBILITY_PROTOCOL.md).
