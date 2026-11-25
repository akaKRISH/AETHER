# 11 | Benchmark and evaluation specification

## AETHER-Mini task suite (proposed, not built)
| Family | Development examples | Held-out evaluation concept |
|---|---|---|
| ML classification | Public digits / tabular baseline | Pinned alternative datasets, hidden split; accuracy and calibration. |
| Algorithm optimization | Sorting or numerical kernels on synthetic inputs | Correctness oracle, randomized inputs, timing on fixed machine. |
| Reproduction | Small documented ML experiment | Recreate metric within preregistered tolerance, fresh replay. |

Only public, appropriately licensed data and safe computational tasks. Start with scikit-learn digits for R0; do not use it as proof of novel AI discovery. Task packs contain baseline, dev split, frozen test hash, metric range/direction, tolerance, resource constraints, challenge cases and reference evaluator. Do not distribute hidden labels in candidate-visible artifacts. Keep benchmark versions immutable. Publish a development pack and a separate held-out protocol; disclose benchmark limitations and potential contamination.

## Metric definitions
| Metric | Definition |
|---|---|
| Verified utility | Normalized task score x external pass (0 or 1). |
| Claim coverage | Claims with all mandatory evidence / claims submitted. |
| False verification rate | VERIFIED claims contradicted by external oracle / VERIFIED claims assessed; N/A if none. |
| Replay pass rate | Passing independent replays / attempted replays. |
| Completion rate | Campaigns reaching valid terminal report / launched campaigns. |
| Budget efficiency | Sum verified utility / total actual tokens or wall-minutes; report separately. |
| Reliability | Timeouts, crashes, policy denials and retries per campaign. |

## Evaluation controls
Measure latency after warmup; use repeated timing samples, fixed hardware, thread settings and input shape. Report medians and spread; distinguish CPU/GPU. Use multiple independent seeds where stochastic. Blind verifier to configuration labels where possible. Challenge fixtures intentionally include incorrect metrics, missing manifests, forged artifacts, test leakage attempts, non-deterministic results and an invalidated hypothesis. A candidate cannot choose the scorer. Human checks a sampled set of claims against raw evidence; report disagreement. Compare only within same budget/hardware regime; report actual usage and cap overruns.

## Release gates
R0: deterministic baseline reruns within tolerance, all required evidence links present, no silent failures. R1: all configurations execute on development pack; freeze held-out rubric; reproduce at least one full study run independently; publish full result table and negatives. Performance improvement is not a release gate for a valid research report; evidence quality is.
