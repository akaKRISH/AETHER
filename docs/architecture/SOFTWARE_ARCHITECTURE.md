# Software architecture

Keep the Rust controller a modular monolith. Pure domain functions calculate transitions, metrics, budget and ranking. Engine use cases validate requests, coordinate I/O and commit state. Storage and execution adapters supply interfaces declared at the application boundary. CLI and later HTTP map typed requests and outcomes to their respective interfaces.

| Behavior | Pattern | Failure outcome |
|---|---|---|
| Domain transition | Exhaustive match on validated state | Typed domain error |
| Workflow | Synchronous use case during initial milestones | Rollback and stable user error |
| Store | Explicit transaction and read model | Reread before bounded conflict retry |
| Launch | Reviewed registry and typed args | Deny unknown task or missing policy |
| Report | Query committed projections and verify referenced content | Mark incomplete or block verified export |

The event journal preserves history, and transactionally updated tables answer common queries. Journal reconstruction differs from rerunning a candidate. Add engine ports when real adapters need them. Keep asynchronous runtimes, filesystem APIs, processes and networking out of core. See [crate ownership](COMPONENT_ARCHITECTURE.md) and [events](EVENT_ARCHITECTURE.md).
