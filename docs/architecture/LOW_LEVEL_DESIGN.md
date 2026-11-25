# Low-level design

The first learning unit is a pure accuracy function over two borrowed label sequences. Validate equal length, nonempty input and allowed label domain before pairing. Count matches and retain exact numerator and denominator. Labels `[0,1,1,0]` and predictions `[0,1,0,0]` give `3/4`, displayed as `0.75`.

| Operation | Input | Output |
|---|---|---|
| `aether init` | Current directory | Idempotent local workspace creation or typed conflict |
| `aether run-fixture` | Initialized workspace and fixed fixture | Run and campaign IDs, exact counts, immutable manifest |
| `aether verify <run-id>` | Existing run ID and recorded inputs | Fixture integrity and replay result |
| `aether report <campaign-id>` | Existing campaign and recorded runs | Read-only deterministic summary |

Phase 1 uses an immutable manifest per run and a stable fixture campaign identity. Use create-new publication with same-directory staging and distinguish abandoned staging from committed data. Validate IDs before lookup. The fixture replay attests to repeated calculation on the recorded fixture. Scientific verification requires its own evidence and independence rules.

Later engine operations include `freeze(config)`, `approve(hash)`, `choose(snapshot)`, `reserve(action)`, `lease(attempt)`, `settle(attempt,observed)` and `project(event)`. Recheck eligibility at reservation and recheck fencing before publication. Commit terminal attempt, settlement and event together.

See [CLI](../design/CLI_AND_RESEARCH_JOURNEY.md) and [source LLD](../archive/v0.2/AETHER/06_LLD.md).
