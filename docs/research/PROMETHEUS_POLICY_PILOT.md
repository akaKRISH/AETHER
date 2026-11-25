# Prometheus development policy pilot

## Frozen development configuration

This pilot exercises the existing `ecar-synthetic-v1` observation model. The configuration prepared before the run uses consecutive development seeds 101–112, the simulator's three task structures, and the same 120 ms simulated wall-time cap and 12-attempt cap for each policy. The configured caps for tokens, output bytes and paid usage are zero. The policy uses the built-in scores and verification reserve. All 36 task-seed pairs run under each of five modes. [Configuration](../../benchmarks/prometheus-policy-pilot.json) · [Simulator](../../crates/aether-engine/src/simulation.rs)

| Prometheus study role | Existing wire mode | Mechanism |
|---|---|---|
| A · Fixed flat policy | B | Explore first and spend a fixed verification allocation. Rank within those windows without downstream weighting. |
| B · Adaptive flat policy | C | Allow exploration and verification to compete. Rank without downstream weighting. |
| C · Fixed graph policy | E | Explore first and spend a fixed verification allocation. Include downstream claim mass in ranking. |
| D · Adaptive graph policy | D | Allow exploration and verification to compete. Include downstream claim mass in ranking. |
| Supplemental one-shot control | A | Choose one exploration and stop. Uses the simulator's common verifier with no language model. |

All policies share the same executable verdict rule, action costs and binary verification observations keyed by task, seed, claim and check. Even the flat arms receive the simulator's common eligibility guard for unsatisfied prerequisites. The graph factor tests a downstream priority term under that guard. It leaves dependency information in the task and verifier. The fixed policy limits verification to its reserved wall-time window. The adaptive policy can spend additional available budget on verification. A contrast between fixed and adaptive policies includes that difference in permissible allocation.

Summarize cumulative utility, verified, rejected and inconclusive claims, actual selections, simulated charges and objective completion. Pair comparisons on each identical task and seed. Report wins, ties and losses and separate the task structures. Run the same configuration in two new processes and compare their report fields. Host duration may vary between runs. Avoid inference about population effects from these simulator fixtures.

## Observed results

On 25 November 2026, two fresh Windows processes produced identical `report` fields across all 180 simulated campaigns. Both used the same release executable, configuration and evaluator. The synthetic outcomes agreed when different modes selected the same action on a task and seed. The step charges summed to the reported usage, and every case stayed within its caps. This is repeatability of the simulator under one executable and host. The task-seed pairs share three designed structures and are unsuitable as independent samples of research performance. [Raw first run](../../research/results/prometheus-policy-pilot.json)

Each row totals 36 cases. Utility has a maximum of 3,600 points and 144 claims per row. Charges and all evidence outcomes in the table come from the simulator.

| Policy | Utility points | Verified / rejected / inconclusive claims | Explore / replay / challenge | Charged wall ms | Objectives met |
|---|---:|---:|---:|---:|---:|
| A · Fixed flat | 1,776 | 76 / 8 / 60 | 116 / 24 / 10 | 2,316 | 12 |
| B · Adaptive flat | 2,542 | 96 / 12 / 36 | 116 / 48 / 10 | 2,748 | 18 |
| C · Fixed graph | 1,776 | 76 / 8 / 60 | 116 / 24 / 10 | 2,316 | 12 |
| D · Adaptive graph | 2,542 | 96 / 12 / 36 | 116 / 48 / 10 | 2,748 | 18 |
| Supplemental one-shot | 1,044 | 36 / 0 / 108 | 36 / 0 / 0 | 576 | 0 |

Each graph-priority contrast tied in utility on all 36 pairs. Graph priority changed the order of selected actions for all 24 pairs with prerequisites and for zero flat pairs. The action identities and charges stayed the same on every pair in both graph contrasts.

Each adaptive-versus-fixed contrast added 766 points across matched pairs, with utility higher on 14 pairs, tied on 22 and lower on zero. The gain was 532 points on the shared-prerequisite cases, 234 on the challenged-prerequisite cases and zero on the flat cases. The adaptive arms selected 24 additional replays, using 432 additional simulated milliseconds under the same caps. The simulator's fixed verification ceiling and common prerequisite guard are part of this result.

The simulator measured 4,816 µs and 4,866 µs of host time for the two whole studies. These times cover the in-process simulation, including scheduling and report construction. They do not estimate the latency of a worker or model, or the scheduling cost in isolation.

## Reproduction and limits

```powershell
cargo build --locked --release --bin aether-lab
.\target\release\aether-lab.exe benchmark benchmarks/prometheus-policy-pilot.json
```

The run used Rust 1.96.1, LLVM 22.1.2 and the `x86_64-pc-windows-msvc` target on an Intel Core i5-13420H. The simulator source matched checkpoint `c017c05`.

| Artifact | SHA-256 |
|---|---|
| Frozen configuration | `9e29ea45c866df83730a5ffed5c83fb59f90fbbafb18ca96ad6db4b8c7a6ff3f` |
| Release binary used | `cec8e0e4acbed1f111ca48a1d5351db6ff0e5c72806300d689e126f816da9bf4` |
| Raw first run | `75e3d79672fb42dd9ff03a2f833182641ba84a69a9facc936919f84a01d13c87` |

The deterministic check generator is also the source of the verdict inputs. It supplies no separate ground-truth labels for a false-verification rate. There was no language model, real experiment outcome, blinded review, calibration set or held-out research task. A larger seed list would still sample the same encoded mechanism. Future matched evaluation needs independent task selection, frozen evidence rules, ground-truth grading, matched measured costs, and a pilot with enough actual tasks to inform sample size. [Benchmark specification](BENCHMARK_SPECIFICATION.md) · [Research integrity and CAVEAT](ORIGINALITY_ASSESSMENT.md)
