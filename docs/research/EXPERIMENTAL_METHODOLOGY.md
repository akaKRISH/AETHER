# Experimental methodology

The study begins at R1, after local evidence mechanics and the approved execution boundary exist. Complete the six-task development pilot across the three proposed families before choosing the main study size. Use scripted candidates for offline systems exercises. Research claims involving LLM-driven agents require approved, identical model versions and a separately authorized budget.

## Setup and freeze

1. Task curator fixes dataset/source licensing, designs the rubric and baseline, pins evaluator and split hashes, validates positive and negative examples, and seals held-out tasks away from the development loop.
2. Pilot the six development tasks. Measure candidate variance, timing instability, resource use, lower and upper cost bounds, evidence diversity, transaction/recovery costs and feasibility of independent reruns. Record every attempt and use pilot variance to plan sample size.
3. Freeze protocol, code, candidate tooling, model revision and sampling settings, per-task caps and reserve, stopping criteria, seed schedule, exclusion policy, selected contrasts and analysis script before opening the held-out set. Timestamp a local Git revision. Public registration needs owner approval.
4. Build a blocked design by task and seed. Randomize configuration order within each block using a separate logged ordering seed. Fresh workdirs and isolated caches prevent cross-run contamination. Hold machine class, thread settings and thermal measurement setup constant or record and stratify approved environment differences.
5. Run the frozen matrix. The source authority's 12 held-out tasks × 3 seeds × 4 configurations = 144 campaigns is provisional. Enabling E at that size adds 36. Avoid treating repeated seeds as independent benchmark tasks.
6. Store all failed, cancelled and incomplete campaigns. Freeze a final export by digest, analyze with the registered plan, and report exceptions, excluded items and alternative analyses openly. Obtain a fresh independent verification of at least one complete study run.

The trusted evaluator is shared across configurations, can consume hidden labels where required, and remains outside candidate view. An external replication and final grader are common controls. In-run scheduling policies differ by design. Time and usage for in-run replay, challenge, controller and search count under the matched campaign caps. Report common assessment costs separately and account for them consistently. The protocol must freeze precisely where the measurement boundary starts and ends.

Review protocol freeze sign-off against [Benchmark specification](BENCHMARK_SPECIFICATION.md), [Reproducibility protocol](REPRODUCIBILITY_PROTOCOL.md) and [Research ethics](RESEARCH_ETHICS.md).
