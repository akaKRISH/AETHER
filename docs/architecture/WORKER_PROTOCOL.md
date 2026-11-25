# Worker protocol v1

The supervisor selects a reviewed executable and argument structure from the registry, creates a fresh isolated working directory, emits one immutable JSON job and exposes declared inputs read-only. The worker receives the job path and publishes one result file plus allowed artifacts.

1. Bind attempt, task and input digest to the persisted lease. Keep fencing credentials in the trusted supervisor.
2. Launch the approved executable with an explicit environment, resource ceilings and fixed arguments. Bound output before parsing.
3. Validate UTF-8 and envelope version, match attempt ID and fence, and verify each declared artifact's size and byte digest. Open only permitted root-relative files through safe handles, reject links, platform aliases, duplicates and unexpected outputs.
4. Treat failure status, nonzero exit, missing result or timeout as an unsuccessful execution. A successful exit and envelope admit the result to evaluation.
5. Score artifacts using the protected evaluator, then let the controller record observed usage and outcome.

Routine worker evidence consists of predictions or declared artifacts. Protocol names, IDs, names and reported digests remain untrusted until checked. Model text cannot supply executable or arbitrary shell arguments. The JSON lifecycle applies to reviewed Rust and optional Python workers. The first fixture executes in process.

See the [schema bundle](../../contracts/README.md), [worker sequence](../diagrams/worker-communication.mmd) and [source native contract](../archive/v0.2/AETHER/21_RUST_ENGINEERING.md).

## Implemented native worker profile

The [native supervisor](../../crates/aether-runner/src/lib.rs) uses `aether.native-worker/1`. It invokes a fixed worker subcommand of the pinned executable. The argument is a bounded JSON job containing `version` and a typed `task`. The output is one bounded JSON envelope with the same version and task and a typed `output`. The supervisor checks the identity and hashes exact output bytes. Trusted reference evaluation runs in the controller.

Each attempt uses a separate empty working directory and clears inherited environment variables. The controller controls time limits, cancellation and output bytes. This profile is for reviewed built-in code and provides no general sandbox or gates for hostile descendants. The generic file-envelope contracts remain the basis for a future independently qualified worker.
