# 10 | Research design and preregistration draft

## Research question and hypotheses
RQ1: At equal resource budgets, does ECAR yield greater verified task utility than a fixed-verification agent? RQ2: Which contribution comes from claim graph dependencies versus adaptive allocation? RQ3: How much overhead does independent replay impose? Primary hypothesis H1: ECAR improves mean verified utility relative to fixed verification. H0: no improvement. Secondary hypotheses: lower false-verified rate and higher verified utility per unit budget. A null or negative result is publishable as an engineering finding, not a failure to hide.

## Design
Pilot 6 development tasks only; calibrate costs, score weights, task tolerances and infrastructure. Main study: provisional 12 held-out tasks across three task families, 3 independent seeds each, four configurations (144 runs). This is an initial feasibility design, not a power calculation. After pilot estimate variance and paired effect, calculate sample size and freeze protocol before opening held-out tasks; expand only if resources allow. Hold model version, tool access, data splits, candidate limits, wall time, token caps and evaluator constant per task. Randomize configuration order within task/seed block; isolate caches, fresh working dirs and independent seeds; same hardware class or stratify by environment. Separate dev and held-out benchmark repos, no tuning on final tasks.

## Configurations
A: single LLM proposal + deterministic execution, no iterative search; external verifier still grades final output. B: iterative agent + fixed replay allowance. C: same agent + flat claim checklist and adaptive scheduling (no graph edges). D: ECAR graph-aware scheduling + same verifier. Optional E: graph + fixed scheduling to complete 2x2 ablation when resources permit. All configurations receive identical grading and hard budgets, not a weaker evaluator for competitors.

## Outcome and analysis
Primary verified utility per campaign = task-specific normalized score in [0,1] multiplied by external pass indicator; zero if required evidence fails or run is incomplete. Mean paired difference D-B with cluster bootstrap over tasks (report 95% CI). Secondary: verified utility per actual token and wall-minute, false verification rate, coverage, replay pass rate and failure categories. Report distribution, effect sizes, uncertainty, all denominators, missing tasks, actual resource usage and protocol deviations. Do not report inflated percentage improvements for zero baselines. Multiple secondary comparisons exploratory; predefine primary comparison. No claims of universal generalization from small benchmark.

## Scientific integrity
Register protocol before main runs (timestamped git commit or OSF if appropriate). Preserve all attempts; no cherry picking, no test leakage. Human review of interpretation, licenses and AI disclosure before public report. Never claim statistical significance without analysis; benchmark thresholds are not observed results. Do not submit AI-generated reports without author accountability and venue policy checks.
