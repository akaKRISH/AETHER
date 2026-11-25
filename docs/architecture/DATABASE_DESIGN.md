# Database design

Phase 2 introduces SQLite through a verified Rust binding. Enable foreign keys for each connection. Keep the DB and WAL on local storage. Backups use a consistent SQLite snapshot plus referenced blobs. Check the upstream WAL advisory and validate the actual linked SQLite source before using WAL. [SQLite WAL](https://sqlite.org/wal.html) · [SQLite backup](https://sqlite.org/backup.html)

| Tables | Keys and invariants |
|---|---|
| project, protocol, campaign | Stable IDs and parent FKs. Protocol version unique per project. Campaign pins immutable config hash and policy. |
| run, attempt | Run belongs to campaign, optional parent run. Attempts have unique identity and current fence. Retry creates another attempt. |
| budget, reservation | Budget keyed by campaign. Reservation keyed by attempt with estimated and settled usage vectors and state. |
| artifact, attempt_artifact, metric | Digest identifies artifact with derived path. Attempt association stores role. Metric records evaluator, split and unit. |
| claim, evidence_edge, claim_dependency, verification | Claim revision and predicate, XOR artifact-or-metric edge, distinct same-campaign dependency pairs, verification rule/evidence/reason and optional replay. |
| event | Unique event ID and campaign sequence, journal append in state transaction. |

Index attempt by run and status, metrics by attempt and name, evidence by claim, and dependencies by target. Validate signed SQLite integer bounds before binding. Store UTC timestamps and use monotonic time for elapsed measurements. Consume no worker-supplied database commands.

Stage blobs, verify exact digest and publish on the same volume before committing references. A crash can leave an orphan. Garbage collection needs retention and a race-safe reference check. Append-only behavior is an application guarantee.

See [entity relationships](../diagrams/entity-relationships.mmd) and [failure recovery](FAILURE_AND_RECOVERY.md).
