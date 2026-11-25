# AETHER native CLI

Build from the workspace root with `cargo build --locked -p aether-cli`. Put the resulting `aether` executable on your command path or invoke it by absolute path from a separate experiment directory. On Windows the file is `target/debug/aether.exe`.

```text
aether init
aether run-fixture
aether verify <run-id>
aether report <campaign-id>
```

Run `init` in a writable experiment directory. It creates `.aether/project.json` and returns the same project/campaign identity on repeated calls. `run-fixture` evaluates fixed binary labels on the CPU and prints exact counts `3/4`, display accuracy `0.75`, a fresh run ID and a manifest SHA-256. Copy that run ID to `verify`, or use the campaign ID with `report` to see validated attempts in stable run-ID order. Replay checks integrity and determinism for the fixed teaching fixture.

## Local format

- The input digest covers exact compact UTF-8 JSON bytes `[[0,1,1,0],[0,1,0,0]]` in fixture version `classification-v1`.
- A manifest records project, campaign and run identity, versioned fixture and evaluator, input digest, exact counts and completed status. Its SHA-256 covers the exact JSON file bytes. The digest appears in the filename under `.aether/fixture/manifests` and in the run reference under `.aether/fixture/runs`.
- Publishing the run reference commits the attempt. Publishing uses a synced staging file and an atomic, non-replacing hard link on the same filesystem. A local filesystem that supports hard links is required. Referenced manifests are never rewritten by the CLI. If a process stops before commit, staged files and unreferenced manifests remain outside the report.
- SHA-256 and replay check corruption and consistency on a trusted local filesystem. The project has no signed provenance or inter-process defense against an owner who can rewrite the whole state directory.

| Exit code | Meaning |
|---|---|
| 0 | Success |
| 2 | Invalid input or a requested ID does not exist |
| 3 | Policy denied project state |
| 4 | I/O or execution failure |
| 5 | Corrupted or incompatible evidence |

See the [milestone and acceptance checks](../../docs/roadmap/MILESTONE_01_NATIVE_FIXTURE.md).
