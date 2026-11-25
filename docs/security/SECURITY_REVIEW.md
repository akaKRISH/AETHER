# Security review

**Scope** — Local Windows Rust release, reviewed built-in worker and authenticated loopback API. The same review applies to the development UI when served from the built local assets.

| Boundary | Implemented controls | Executed evidence |
|---|---|---|
| Native state | Validated IDs, bounded reads, staged publication and exact-byte SHA-256 checks | Native tests exercised repeated and concurrent init, abandoned staging and changed persisted bytes |
| Frozen campaign | Protocol digest includes the approved inputs, evaluator and policy. The controller checks the digest and executable before running | Integration tests rejected an incorrect approval and a changed protocol before report export |
| Reviewed worker | Fixed executable and registered subcommand, fresh empty working directory, cleared environment, bounded request/output, deadline, cancel flag and trusted scoring | Tests covered the reviewed sorting variants, malformed version, wrong evidence and cancellation before launch. Release campaign ran three separate processes |
| Evidence and budgets | Transactional attempts, protected verification reserve, stale lease checks, scoped supervisor recovery, replay-gated verdicts and transitive revocation | Store tests exercised rollback, idempotence, tamper detection, lease expiration, budget overrun and dependency propagation. API recovery test covered a stranded campaign |
| Loopback HTTP | Binds IPv4 loopback, local bearer authentication, protected mutations, bounded JSON, sanitized errors and attachment downloads | API tests exercised unauthenticated reads and stream denial, approval sequencing, SSE resume, repeated draft cancellation and authenticated artifact retrieval |
| Model and literature | Strict sorting enum output, deterministic default adapter, local model opt-in, Crossref explicitly disabled by default | Tests rejected model-supplied execution parameters, checked fixture parsing and denied default external requests |
| Supply chain | Locked dependencies and published advisory databases | [Cargo and npm audit](../governance/DEPENDENCY_INVENTORY.md) |

The review found a shutdown path that left live streams and workers active and a recovery path that could leave an interrupted campaign stuck in Running. Shutdown now signals active cancellation and closes streams. The controller takes a process-scoped campaign lock and exposes scoped recovery after leases expire. The integration suite exercised interruption recovery. A Windows Ctrl-C drill with an authenticated idle stream open closed the stream and exited successfully in 0.458 s. [Validation](../testing/TEST_RESULTS.md)

## Residual risks and boundaries

- This profile relies on trust in the installed application and its reviewed built-in code. Time and output bounds are implemented. Enforced operating-system memory, CPU, child-tree and network isolation remain unqualified. Execution of externally supplied binaries or generated code stays disabled.
- The loopback token represents the local owner. Hosted identity, remote TLS, multi-tenant fairness and delegated roles require a separate approved deployment design. Keep the token in the process environment and never check it into the repository.
- A same-user host adversary can replace the executable or mutate both storage and digests. The current digests detect changed bytes relative to pinned expected values. They provide no externally signed chain of custody.
- A passing replay establishes the frozen result on two processes using the pinned evaluator and inputs. Publication and independent scientific review require a stronger protocol.
- The local restore drill produced an identical, digest-checked campaign report. Interruption during an active worker and process isolation denial tests on a supported external profile remain open. [Restore evidence](../testing/TEST_RESULTS.md)

[Threat model](THREAT_MODEL.md) · [Permission model](PERMISSION_MODEL.md) · [Test results](../testing/TEST_RESULTS.md)
