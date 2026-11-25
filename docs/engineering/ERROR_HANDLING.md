# Error handling

Use typed domain and boundary errors. Propagate context about the operation and stable resource identity. Keep sensitive paths, credential material, hidden labels and raw worker diagnostics in appropriately protected channels. A crash report preserves enough information for recovery without dumping untrusted input.

| Class | CLI exit | Handling |
|---|---|---|
| Success | 0 | Emit machine-readable or human result |
| Validation and missing requested ID | 2 | Reject before writes where possible |
| Policy or authorization denial | 3 | Deny and audit scoped reason |
| I/O or execution failure | 4 | Distinguish timeout, OOM, cancellation, infrastructure and evaluator error |
| Integrity failure | 5 | Block evidence use, quarantine affected material and preserve reason |

Treat errors as application results even for expected worker failures. A transient infrastructure failure may get one new attempt under policy after safe reconciliation. Invalid candidates, policy denials, integrity and budget failures retain their actual outcomes. A missing or invalid mandatory replay results in an inconclusive claim when no frozen rule establishes a rejection.

See [source SRS failure semantics](../archive/v0.2/AETHER/04_TRD_SRS.md) and [recovery](../architecture/FAILURE_AND_RECOVERY.md).
