# Event architecture

Commit authoritative facts and journal events in one transaction. An envelope includes schema version, event ID, campaign ID, campaign sequence, type, UTC timestamp, trusted actor, correlation ID and payload. Sequence orders events within the campaign independently of timestamps.

| Family | Example | Payload |
|---|---|---|
| Protocol | protocol.approved | Revision, approved hash and policy |
| Budget | budget.reserved / budget.settled | Attempt, reservation, unit-tagged vectors and attribution |
| Run | run.leased / run.started / run.completed / run.failed | Run and attempt IDs, timings and error class |
| Claim | claim.proposed / claim.verified / claim.rejected / claim.revoked | Claim revision, rule, evidence and reason |
| Campaign | campaign.cancelled | Request identity and budget disposition |

The [event schema](../../contracts/schemas/event.schema.json) covers the envelope. Implement per-type payload validation before use. Gate unknown writer versions. Consumers reauthorize every subscription, resume after sequence and deduplicate by event ID. External notifications indicate committed rows are available. If retention invalidates a cursor, provide a snapshot refresh. Worker status never becomes an authoritative event without controller checks.

See [source events](../archive/v0.2/AETHER/09_API_EVENTS.md) and [recovery](FAILURE_AND_RECOVERY.md).
