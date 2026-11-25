# System architecture

The controller, ledger and evaluator are trusted. The runner mediates worker execution. Workers receive reviewed task inputs and resources. Hidden evaluation labels reside in a separate trusted location. Optional model suggestions enter through a schema-validating adapter.

| Boundary crossing | Guard |
|---|---|
| Operator → controller | Current project, approved protocol hash, policy and budget |
| Proposal → controller | Schema, registry and policy |
| Supervisor → worker | Approved executable, isolation profile and bounded job |
| Worker → controller | Root-confined outputs, size, byte hashes, attempt ID and lease fence |
| Evaluator → ledger | Frozen protocol, lineage and requisite independent evidence |

A campaign pins one immutable protocol and policy. The engine ranks against a committed snapshot, then rechecks admission when reserving. Results are fenced and settled with the same transaction as the attempt outcome and journal append.

Local reports read committed evidence and explain missing links. Windows native hosts the trusted fixture. Candidate worker support requires a qualified isolation environment. See [deployment](DEPLOYMENT_ARCHITECTURE.md) and [trust diagrams](../diagrams/README.md).
