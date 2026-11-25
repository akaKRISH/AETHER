# Scalability design

Start with one worker, one local controller and transactional local state. Profile scheduler decisions, writes, run time and report traversal independently.

| Measured constraint | Proposed response | Proof |
|---|---|---|
| Evidence lookup dominates | Index dependencies, then consider cached read model | Cycle rejection and revocation propagation |
| Read/write contention | Bounded reads and SQLite WAL | Snapshot and restore checks |
| Concurrent API writers exceed local profile | PostgreSQL behind engine port | Same admission and settlement semantics |
| CPU throughput limits validated tasks | Measured concurrency increase or qualified isolated workers | Aggregate caps, stale fence and cancellation tests |
| Artifacts exceed local storage | Immutable object storage | Integrity, consistency and retention tests |

Future remote workers use authenticated transport and short-lived authorization scoped to one attempt and execution profile. The controller fences stale workers and remains ledger owner. Hold labels in the trusted evaluation scope. Explicitly approve provider, data, budget and network. Test partition recovery and reject late outcome before enabling remote. See [distributed topology](../diagrams/distributed-workers.mmd).
