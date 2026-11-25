# Resource management

Use explicit nonnegative integer units. Campaign caps cover wall time in milliseconds, attempts, tokens and cost in micro-USD. Worker ceilings cover thread slots, memory, output bytes, process count and timeout. Peak memory is a quota and capacity check. Freeze whether wall-time allocation means campaign elapsed time or sum of charged worker times.

| Action | Atomic behavior |
|---|---|
| Reserve | Check `used + reserved + estimate <= cap` for every capped dimension using checked arithmetic, then persist reservation and queued attempt together |
| Lease | Fence one queued attempt with expiry and supervisor identity |
| Settle | Validate current lease, post actual or conservative usage once, release outstanding reservation and append events |
| Retry | New attempt and reservation with prior cost retained |
| Overrun | Persist observed excess and deviation, halt new admissions and allow safe cleanup |

Exploration preserves configured verification headroom. A zero cap permits only zero consumption. Proposed pilot limits are 30 minutes, 20 attempts, 60,000 tokens, zero paid spend. Reviewed CPU workers initially target one worker, 2 threads, 2 GiB memory, 5 minutes and 200 MiB output. Calibration and frozen protocol decide final study limits. Phase 1 uses zero model tokens.

Use monotonic timing for in-process elapsed intervals. Ambiguous crash usage is conservatively charged and flagged. Delayed external provider reports can exceed estimates. Record the actual excess and stop admission.

See [budget sequence](../diagrams/budget-ledger.mmd) and [source requirements](../archive/v0.2/AETHER/04_TRD_SRS.md).
