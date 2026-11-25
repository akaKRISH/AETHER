# Observability

Attach campaign ID, attempt ID, monotonic elapsed duration, phase, correlation ID, UTC event timestamp and typed reason to each transition. Report charged and reserved quantities by unit. Include CPU and memory statistics when supported, and represent unavailable samples explicitly. Separate structured audit events from bounded diagnostic logs.

Flag stalled leases, usage overruns, denied operations, missing evidence and failed backups. Redact paths and sensitive data before any export. Measure consistency between audit projection and emitted summaries. [Test plan](../testing/TEST_PLAN.md)
