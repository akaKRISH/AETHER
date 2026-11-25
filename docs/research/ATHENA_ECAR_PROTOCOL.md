# Athena ECAR diagnostic protocol

## Development design

This synthetic diagnostic investigates the equal verified utility in all matched Prometheus graph-priority comparisons. Competing explanations are spare budget and saturation, weak topology, optimistic downstream mass on an unreliable prerequisite, and verifier noise or worker failure. Ranked ordering is a separate outcome from selection and final utility. [Prometheus pilot](PROMETHEUS_POLICY_PILOT.md)

Freeze the [development configuration](../../benchmarks/athena-ecar-development.json) before inspecting the outcomes. It crosses five task families, three wall-time caps and 24 seeds. The flat case is a negative control. The valuable hub and deep chain test prioritization of enabling steps. The unreliable hub tests a priority term that ignores the chance that the hub is wrong. The noisy, costly case tests bad observations and failed attempts. All cases enter the report.

The four arms share costs, caps, true claim states, replay signals, challenge signals, execution failures and verdict rules within each task-seed-cap pair. Truth is a keyed draw and depends on ancestors. The observed checks flip this truth according to separate keyed draws. Replay and challenge cannot mature a dependent claim without observed verification of its prerequisite. One simulated failure terminates work on that claim, so failed work still costs budget. A false verification is a claim that passes the required observed checks with false hidden truth. [Diagnostic code](../../crates/aether-engine/examples/ecar_diagnostics.rs)

The fixed arms explore first while exploration is affordable and limit verification to half the wall-time cap. The adaptive arms allow the same exploration and checks to compete under the quarter verification reserve. The graph arms add descendant weights to the priority score, while the flat arms use direct weight. All arms use the same prerequisite checks. Their graph contrast tests incremental priority within a common evidence rule.

## Analysis fixed before outcomes

Primary contrast is ECAR minus adaptive flat evidence. Divide verified true utility by the total claim weight, multiply by 100 and average across families and caps within each seed. Give every seed equal weight. Report the mean paired difference, its sample standard deviation, and a percentile interval from 2,000 bootstrap resamples of whole seed blocks using RNG seed 617. Report individual family rows, true verified utility, false verification counts and rates, coverage, unresolved high-impact claims, replay passes, failure counts, and cost per true verified claim. Calculate cost using total simulated wall charges divided by successful true verifications. A zero denominator yields an absent ratio. Timing separates the observed host scheduler call total from simulated cost.

Report graph-aware fixed versus fixed flat and adaptive versus fixed as secondary descriptive contrasts. For planning, compute the normal-approximation sample size `ceil((1.96 + 0.84)^2 * SD^2 / 5^2)` with a minimum of 30 seed blocks. Five normalized points is the planning effect, 0.05 is two-sided alpha, and 0.80 is power. Treat this as a sizing heuristic conditional on these development families. It cannot size a scientific campaign with different data.

## Held-out gate

Freeze evaluation input bytes and their SHA-256 before a formal held-out run. Record the policy source revision, evaluation procedure, label custody, seed-block independence and reviewer approval. Keep formal outcomes sealed while changing the policy and preserve the development data, including negative outcomes. A separate independently chosen task set with real executable evaluations is required before claiming scientific generalization. This diagnostic simulation supplies a mechanism study under its explicit assumptions.

The development configuration SHA-256 is `76c9b935fb52b3b84eddc4afdb350fbe103e479ccf37747b9f69196e2434675b`. The [frozen synthetic held-out specification](../../benchmarks/athena-ecar-heldout-spec.json) uses 30 disjoint seed blocks and revised task parameters. Its SHA-256 is `876cbba77e9746dca4bab1212890e3c1bc57c19a82ab047e8fa9a5362fce1e00`. The pilot estimate reached the predeclared minimum of 30 blocks. The current runner accepts development files. Formal evaluation and independent reviewer sign-off remain pending, and this public file does not establish independent label custody.

## Measured development results

Two fresh processes reproduced 1,440 outcome rows exactly. The ECAR-minus-adaptive-flat estimate was +0.173 normalized utility points (seed-block SD 2.575, 95% percentile interval −0.840 to +1.188). Among 360 pairs, 62 improved, 238 tied and 60 worsened. The pooled estimate conceals a +5.312-point difference in the valuable hub and a −3.935-point difference in the unreliable hub. The deeper chain averaged −2.209 points. The flat control tied. These observations are consistent with downstream mass sometimes prioritizing a useful enabling check and sometimes spending resources on an unreliable prerequisite.

| Arm | Mean utility % | Verified / false verified | Coverage | Unresolved high-impact | Replay passes / attempts | Simulated ms per true verification |
|---|---:|---:|---:|---:|---:|---:|
| Fixed flat | 13.184 | 400 / 0 | 23.85% | 991 | 687 / 962 | 37.00 |
| Adaptive flat | 18.419 | 549 / 3 | 30.83% | 907 | 794 / 1,109 | 28.71 |
| Fixed graph | 12.500 | 387 / 1 | 23.65% | 974 | 696 / 988 | 38.46 |
| ECAR | 18.592 | 582 / 3 | 33.61% | 877 | 854 / 1,221 | 27.27 |

The host ran the pilot in 113.636 ms and spent 39.962 ms inside scheduler calls. Simulated per-campaign wall charges are task inputs. The false-verification rates use verified claims as denominator and known hidden truth as label. The related bounded sorting campaign uses a deterministic trusted evaluator and separate worker processes. [Raw development outcomes](../../research/results/athena-ecar-pilot.json) · [Paired summary](../../research/results/athena-ecar-summary.json)
