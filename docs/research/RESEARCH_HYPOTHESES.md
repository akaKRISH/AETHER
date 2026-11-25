# Research hypotheses

| ID | Hypothesis | Validation | Falsification or qualification |
| --- | --- | --- | --- |
| H1 | ECAR D improves mean verified utility relative to fixed replay B at equal caps. | Primary paired D − B comparison by task and seed with task-cluster interval. | A point estimate at or below zero opposes the directional premise. An interval spanning zero leaves improvement unresolved. |
| H2 | Dependency-aware scheduling improves verified utility relative to flat adaptive scheduling. | Compare D − C, controlling candidate proposer, hard budget, evaluator and all non-graph observations. Inspect work spent on distinct pivotal claims. | Equal or lower utility, excessive cost or improvement confined to a known task family weakens the intended benefit. |
| H3 | Independent replay and challenge information reduce externally confirmed false verification. | Compare oracle-assessed false verification rates and counts under frozen checks. Compare rates jointly with the number of verified and inconclusive claims. | An empty verified denominator provides no rate estimate. Stronger abstention alone supports a different conclusion. |
| H4 | Extra ledger and replay work can be quantified and can coexist with better verified utility per actual resource consumed. | Report CPU, wall time, memory and tokens for all configurations, plus separate matched controller overhead measurements. | A scheduler may gain utility at disproportionate overhead or lose headroom under a constrained regime. Report the trade-off by dimension. |

H1 has the only confirmatory comparison in the proposed first main study. H2–H4 are secondary and treated as exploratory until a protocol specifies independent confirmation. The attribution question is strongest with all four factorial cells B–E. See the [statistical analysis plan](STATISTICAL_ANALYSIS_PLAN.md) and [ablation design](ABLATION_STUDY_DESIGN.md).
