# aether-store

The local trusted Rust controller uses this crate for transactions, frozen campaign identity, leases, integer budget accounting, and scientific evidence. A transaction commits every related write or leaves the prior state intact. For example, a completion binds the metric, manifest, artifact, usage and audit sequence in one write transaction.

## Run and evidence

1. Create a campaign with pinned protocol, policy and evaluator digests. Approve the protocol digest and enter `running` before admitting attempts.
2. Register an experiment and a logical run. A replay belongs to a new run with a parent and the same experiment, seed and input digest.
3. Reserve budget and start a fenced attempt together. Complete with a bounded JSON manifest, trusted metrics and observed resource use. Sending the identical completion again leaves the result and budget unchanged.
4. Link metrics or artifacts to claims. The trusted verifier records a verdict under the frozen rule and evaluator. When replay is required, a successful distinct replay and an accepted replication comparison are required. Revocation changes current projections and appends events while preserving earlier verdicts.

## Local limits

Schema 1 uses SQLite foreign keys, checked Rust integer conversions, rollback journals and full synchronous writes. It stores raw bytes as SHA-256-addressed SQLite BLOBs. This keeps database and blob commits atomic for bounded local experiments. Each attempt accepts up to 8 MiB of artifact bytes, 32 artifacts, 64 metrics and a 256 KiB manifest. Larger workloads can move immutable bytes to a separate store while preserving digests, with recovery and backup qualification at that stage.

Audit and campaign lists allow pages of up to 512 records. Report snapshots cap each data kind at 4,096 records and reject overflow. Campaigns admit at most 512 claims. Lease recovery reconciles up to 512 expired attempts in a transaction and charges the estimate conservatively. The controller is responsible for stopping processes before calling recovery. Protect the SQLite file and its directory as trusted local state. An audit log records what the trusted controller wrote and gives no cryptographic guarantee against an operator with raw file access.

Run `cargo test --locked -p aether-store` for the rollback, replay, graph, budget and persistence tests. [Architecture](../../docs/architecture/README.md) · [SQLite decision](../../docs/decisions/ADR-002.md)
