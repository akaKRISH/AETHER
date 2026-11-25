# Resume AETHER

## Checkpoint and phase

Baseline `c017c05` is the last independently confirmed clean foundation. The Prometheus recovery commit preserves the current implementation on `main`. The active phase is Windows release verification and integration closeout.

## Preserved implementation

- Frozen sorting and maximum-subarray pipelines, reviewed worker admission, journal snapshots, replay, and evidence export live in `crates/aether-api`, `crates/aether-runner`, and `crates/aether-store`.
- Model and literature adapters are in `crates/aether-api/src/planning` and `crates/aether-api/src/literature`. Four bounded knapsack arms and Criterion harnesses are in `crates/aether-engine` and `crates/aether-store`.
- The new browser controls, test harnesses, research dumps and delivery docs are in `web`, `tests`, `research/results`, and `docs`. The original sorting report SHA-256 remained `55ef0a797a07c7ce2e01538c3dd67e7e030621e14cd8b2bb19949cd8e1159b08`.

## Latest verification

The baseline rerun passed 33 Rust and three frontend tests. The expanded pre-fix integration passed 49 Rust tests, formatting, strict Clippy, native release, frontend lint/type/test/build, Chromium desktop/mobile smoke, and Criterion. The recorded optimization report verifies sums 623 and 1,131 on the frozen inputs and retains two counterexamples to the rejected singleton claim. The simulated policy pilot and measured knapsack checks have tracked exports.

## Exact next action

Build the release with the minimal Windows NVML environment fix and run `aether-lab hardware`. Before that patch, a cleared child environment with only `SystemRoot` produced `Failed to initialize NVML: Unknown Error`. The direct driver probe succeeded with `SystemRoot` and `ProgramFiles`. If the release passes, capture final timing and driver observations, check browser workflows, then complete the audit records and local checkpoint.

## Remaining dependencies

Live model and bibliographic retrieval require an authorized ready endpoint or explicit outbound query. The preflight found no listener and 1.30 GiB free host RAM. GPU inference, stronger worker isolation and an independently controlled scientific evaluation have separate readiness gates. [Continuation](NEXT_ACTIONS.md) · [Test results](docs/testing/TEST_RESULTS.md)
