# Failure and recovery

Persist queue entry and reservation before dispatch. Use a persisted expiry and fence to control publication and an executor-side timeout for independent termination. Work may execute more than once. Each attempt can commit and settle only once.

| Failure window | Recovery |
|---|---|
| Before reservation commit | Roll back with no dispatchable attempt |
| Commit, before launch | Recover queued entry or cancel and release |
| Lease or launch, before output | Fence and reconcile executor, conservatively charge if unknown |
| During artifact stage | Quarantine and respect retention before cleanup |
| Blob published, DB commit missing | Blob remains a grace-based cleanup candidate |
| Outcome committed, caller disconnected | Return durable result by attempt/idempotency identity |
| Stale outcome | Reject publication and preserve current lease |
| Invalid evidence | Record failure, block verified export and append invalidation if needed |

Only transient infrastructure errors qualify for one automatic retry, with a fresh attempt and reservation. Shutdown blocks admission and reports cleanup pending. Test safe interruptions around each commit boundary and verify one terminal record and settlement. Restore from a consistent database snapshot with every referenced artifact.

See [recovery diagram](../diagrams/failure-recovery.mmd), [source SRS](../archive/v0.2/AETHER/04_TRD_SRS.md) and [database design](DATABASE_DESIGN.md).
