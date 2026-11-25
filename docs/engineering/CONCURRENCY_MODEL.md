# Concurrency model

Start with synchronous single-worker execution. A controller may serve multiple requests later, while a transaction still decides whether an attempt is admitted. Concurrent reads and a single SQLite writer can coexist under a qualified WAL configuration. Hold no open write transaction across worker launch, evaluation, artifact streaming, network calls or any future await.

1. Obtain snapshot and rank feasible candidates outside the write transaction.
2. Begin write and reread approval, fence eligibility and resource caps. Atomically reserve and enqueue.
3. Commit a lease before dispatch. Runner observes cancellation and uses a bounded timeout independently.
4. Atomically settle and terminalize under the current fence.
5. If an asynchronous runtime becomes necessary, use bounded channels and explicit shutdown ownership.

Use sequence and lease tokens for ordering. Wall-clock timestamps alone cannot order distributed work. Define backpressure when queued work, output acquisition and result validation contend. When a cancellation races completion, committed leasing/state order determines the accepted terminal action. Test reserve/settle contention, repeated cancel, stale result and interruption between staging and commit.

See [resource management](../architecture/RESOURCE_MANAGEMENT.md) and [recovery](../architecture/FAILURE_AND_RECOVERY.md).
