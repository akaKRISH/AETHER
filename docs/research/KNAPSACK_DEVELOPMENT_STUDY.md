# Knapsack development study

The reviewed task generates 12 items from a recorded seed. Each item has a positive integer weight and value. A solution chooses a subset whose weight fits within one third of the total weight. The report freezes the generated items and integer capacity. Its objectives are higher total value, then lower weight on ties.

## Matched search arms

| Arm | Proposal source | Verification schedule |
|---|---|---|
| Random | Seeded pseudo-random 12-bit masks after the empty baseline | Final slot |
| Evolution | Draw initial masks, select a retained feasible parent and flip one bit, and periodically draw a new mask | Final slot |
| Fixed verification | Same evolutionary proposal policy | Every fourth slot and the final slot |
| ECAR guided | Same evolutionary proposal policy | Trusted scheduler prioritizes screening or checking a changed incumbent using elapsed slots and candidate priority. The final slot checks the incumbent. |

Each arm gets the same 8–96 abstract evaluation slots, one initial empty-subset evaluation and a reserved final check. An evaluated proposal and an independent check each consume one slot. The slot cap supports comparison of allocation behavior. The measured wall time for a complete run also includes the exhaustive oracle and report construction in memory.

The search records each mask, parent slot, feasibility, weight, value and whether it improved the incumbent. An eight-member pool retains feasible, distinct high-ranked parents. Periodic random proposals introduce candidates outside that pool. Repeats and infeasible candidates remain in the archive. The scheduler receives bounded actions named `propose` and `verify-incumbent` and records its actual decisions.

## Independent checks and interpretation

The proposals use a bitwise mask to represent selected items. The candidate evaluator iterates the item array and accumulates included weights and values. The verifier separately walks only the set bits and recomputes both totals. A second reference walks inclusion and exclusion branches and finds the exact optimum by exhaustive enumeration, including the least weight among optimal subsets. The reference affects outcome grading after search and stays outside proposal and scheduling decisions.

Verification confirms that the reported incumbent's weight and value recompute and its weight fits. Regret equals the exhaustive optimum value minus the verified incumbent value. Positive regret preserves a useful failure to match the optimum. Verification here uses separate code within the same Rust process. Independent samples for a future comparison would come from preregistered seeds or different task instances. Four arms on one seed share the same instance and are paired observations.

The development study exercises proposal ancestry, diversity, rejection and verification allocation. Estimates for ECAR are local policy choices. A controlled study would need a frozen held-out instance set, measured compute caps, and an evidence-graph ablation. [Benchmark specification](BENCHMARK_SPECIFICATION.md) · [Originality assessment](ORIGINALITY_ASSESSMENT.md)

## Read and rebuild

Start in `evolve`, `search`, `score`, `check` and `exhaustive_optimum` in the [implementation](../../crates/aether-engine/src/evolution.rs). A mask of `0b101` selects the first and third items. Flipping the second bit creates `0b111`, a child that also selects the second item. Addition combines the selected item weights and values. A child can exceed the capacity even when its parent fits.

Try three items with `(weight, value)` pairs `(3, 7)`, `(4, 8)` and `(2, 4)` under capacity 6. Enumerate all eight masks and check which feasible mask has the highest value. Rebuild the scorer in the [standalone learning workspace](../../learning/README.md), then explain why feasible verification and finding the mathematical optimum are separate questions.
