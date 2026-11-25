# Coding standards

Choose a direct function or existing type before adding a behavioral abstraction. Map external values to typed domain inputs at the trust boundary. Use explicit units in names and prefer readable error messages and enums. Store scientific rules in their authoritative modules, not repeated CLI branches. Keep output order deterministic for observable records.

Apply the concise-code standard to Rust, Python, JavaScript, TypeScript, React, SQL, PowerShell, shell scripts, tests and infrastructure configuration. Prefer a compact idiomatic expression when it is equally correct, safe and clear. Remove redundant variables, copies, serialization, queries and wrappers before adding another layer. Keep meaningful types and identifiers. Give detailed logic room where it protects numerical stability, concurrency, evidence, error context or accessibility. Let repository formatters lay out the code. Review related code during implementation and preserve stable public and research behavior. Use actual observations to support performance claims.

| Review item | Example check |
|---|---|
| Identity | Attempt ID cannot be mistaken for campaign ID |
| Integrity | Invalid manifest yields typed error before replay/report |
| Arithmetic | Budget additions and numeric conversions are checked |
| Side effects | Pure evaluation does not open files or mutate policy |
| Reviewability | A small change has an executable success case and decisive edge case |
| Maintainability | Comment explains an invariant, subtle failure or deliberate limitation |

Prohibit network, process and wall-clock assumptions in deterministic core. Public methods document preconditions and failures. Use controlled synthetic fixtures for unit and integration tests. Review error paths for partial state and path handling before sending decisions upstream.

See [error handling](ERROR_HANDLING.md) and [the Rust standard](RUST_ENGINEERING_STANDARD.md).
