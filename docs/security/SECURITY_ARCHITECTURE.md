# Security architecture

The Rust controller is the authorization point. It validates a pinned protocol, checks the owner’s approval and the action’s current budget, then passes a constrained job to the runner. The runner receives an approved executable identity and typed input. The trusted evaluator consumes bounded predictions and protected labels in a separate context. Only the controller may apply the evaluator verdict to the ledger. The artifact store validates and identifies content by digest. A digest detects change against a trusted reference and does not authenticate the origin by itself.

| Boundary | Data allowed | Authority retained |
| --- | --- | --- |
| User or proposer to controller | Typed requests and untrusted rationale | Controller binds authorization to protocol, policy and caps |
| Controller to runner | Approved job identity, input digests and resource allowance | Controller controls lease, budget and lifecycle |
| Runner to ingestion | Bounded predictions and declared artifacts | Ingestion resolves paths and validates type, size and digest |
| Ingestion to evaluator | Validated inputs and scoped private label access | Evaluator computes the outcome under frozen rules |
| Report to export | Reviewed evidence subset and licensing metadata | Owner authorizes publication or transfer |

The Windows fixture runs trusted deterministic Rust with no candidate process. Optional workers require a qualified isolation profile and a separately approved task pack. Future controller-model adapters may make scoped network requests after approval. Candidate execution remains offline under the initial worker design. Architecture coverage and evidence are tracked in the [threat model](THREAT_MODEL.md).
