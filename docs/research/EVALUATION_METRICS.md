# Evaluation metrics

| Metric | Definition and measurement boundary |
| --- | --- |
| Verified scientific utility | For each campaign, frozen normalized task utility in [0,1] times the external pass indicator. Invalid or incomplete campaigns contribute zero in the primary analysis. Normalize each task before comparing across families. |
| Claim coverage | Submitted claims with all mandatory links and valid artifacts divided by submitted claims. Also report protocol-required claim slots covered to expose selective omission. With zero submitted claims, report N/A for the submitted-claim rate. |
| False verification rate | Externally contradicted VERIFIED claims divided by VERIFIED claims assessed by the independent oracle. Report number assessed, total verified and assessment design. An empty assessed denominator is N/A. |
| Replay pass rate | Passing independent replays divided by attempted replays. Failed and incomplete replay attempts remain in the denominator. Report N/A when no replay was attempted. |
| Experiment completion | Campaigns reaching a valid terminal report divided by launched campaigns. Disclose verdict breakdown and failure categories separately. |
| Efficiency | Sum of verified utility divided by total actual tokens and by actual wall-minutes, separately. Report counts and raw sums. A zero denominator yields N/A. |
| Wall time and CPU | Monotonic elapsed milliseconds for the declared campaign boundary, CPU time of supervised processes and average utilization relative to the configured capacity. Label whether utilization can exceed 100% on multicore by convention. |
| GPU and memory | GPU utilization/device time where available, peak worker RSS and controller RSS separately. Mark unavailable readings N/A with reason and disclose sampling interval and units. |
| Cost and tokens | Metered input and output tokens by model operation and compute estimates by frozen tariff. Distinguish measured payments, modeled costs and free credits. Represent authoritative budget amounts in integer units. |
| Reliability and recovery | Timeout, execution failure, policy denial, cancellation, retry and deduplicated commit counts, plus detection and recovery duration for classified incidents. |

Predefine the task-specific utility mapping, failure handling and the independent oracle before main runs. A signed or hashed output only authenticates its bytes or origin. Verify the content against the evaluator and required provenance. Use median and spread for repeated timing observations. Pair scores on the same task and seed and compute uncertainty by task clustering as described in the [statistical plan](STATISTICAL_ANALYSIS_PLAN.md).
