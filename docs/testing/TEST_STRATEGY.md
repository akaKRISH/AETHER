# Test strategy

Start with pure unit tests for counts, boundary inputs, state transitions and deterministic ranking. Exercise graph acyclicity, budget conservation and settlement idempotence with generated cases once implemented. Use integration tests for persistence, lifecycle and artifact checks, followed by one end-to-end CLI path across fresh processes. Add worker-specific tests on an explicitly qualified isolation runtime.

Use realistic failure fixtures at the boundary under test and a fresh local state per test. Confirm that failures avoid partial committed effects and preserve earlier verified evidence. Planned branch coverage for policy, budget and ledger is ≥80%, with every critical invariant covered. [Test plan](TEST_PLAN.md)
