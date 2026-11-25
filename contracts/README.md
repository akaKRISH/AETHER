# Design contracts

These are versioned drafts. Controller and supervisor enforcement belongs in the approved implementation.

| Interface | Schema | Example | Authority |
|---|---|---|---|
| Claim proposal 0.1.0 | [schema](schemas/claim.schema.json) | [example](examples/claim.example.json) | Evidence references and predicates proposed to the controller |
| Event envelope 0.1.0 | [schema](schemas/event.schema.json) | [example](examples/event.example.json) | Trusted writer only; add per-kind payload validation before emitting |
| Protocol 0.2.0 | [schema](schemas/protocol.schema.json) | [example](examples/protocol.example.json) | Owner-approved version and caps; classifier example rule only |
| Worker job 1.0 | [schema](schemas/worker-job.schema.json) | [example](examples/worker-job.example.json) | Supervisor creates from matching lease and approved task |
| Worker result 1.0 | [schema](schemas/worker-result.schema.json) | [example](examples/worker-result.example.json) | Untrusted artifact declaration; validated then independently evaluated |

The protocol draft uses milliseconds, bytes and micro-USD. Numeric quantities in this portable draft stay within exact nonnegative JSON safe-integer bounds. Runtime validation enforces cross-field bounds, approval, permitted resources, attempt/fence identity, artifact path safety, references, duplicate-key rejection and semantic rules. Agree on versioned canonicalization before exchanging canonical JSON digests. Artifact digests refer to exact bytes.

Example digests are synthetic. Examples illustrate shape and confer no authority to run a task. Preserve original source placeholders and version series in the [archive](../docs/archive/v0.2/AETHER/contracts/README.md). The [worker protocol](../docs/architecture/WORKER_PROTOCOL.md) explains the process boundary.
