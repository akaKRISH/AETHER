# Failure injection

Use temporary test data. Interrupt after staging and before commit. Inject a stale lease, repeated settlement, malformed envelope, bad checksum and an exhausted quota. Assert the earlier outcome remains readable, new state is correct, and no duplicate accounting occurs. Exercise cancellation with a benign looping fixture and child process on the supported runner. Validate deterministic cleanup and references after restart.

Record which step failed, disposal of scratch artifacts, elapsed time and recovery/state projections. [T04, T05, T08–T11, T18–T21](TEST_PLAN.md)
