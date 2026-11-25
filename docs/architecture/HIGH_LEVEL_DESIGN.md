# High-level design

AETHER is a local scientific research controller with a native Rust CLI, deterministic domain core and reviewed execution lane. One trusted controller owns policy, admission, budgets, evidence, verdicts and reporting. The execution supervisor brokers reviewed task attempts. Candidates emit predictions and artifacts. Trusted evaluation scores them with protected data.

| Flow | Decision or output |
|---|---|
| Owner → controller | Approve frozen protocol hash, policy and caps |
| Core → engine | Ordered eligible action or explicit stop reason |
| Engine → store | Atomic reservation and queued attempt |
| Runner → worker | Reviewed executable, immutable inputs, bounded output |
| Runner → evaluator | Validated artifacts and observed resource usage |
| Evaluator → engine | Trusted measurements and rule outcome |
| Engine → store → report | Committed decision, lineage and exportable outcome |

The first milestone is a built-in CPU classification fixture plus local immutable manifests and deterministic replay. Later phases add SQLite, reviewed workers and ECAR. HTTP, browser and remote compute have separate gates. See [component boundaries](COMPONENT_ARCHITECTURE.md), [architecture diagrams](../diagrams/README.md) and [source architecture](../archive/v0.2/AETHER/05_ARCHITECTURE.md).
