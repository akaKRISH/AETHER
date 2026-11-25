# ECAR mathematical foundations

The policy approximates a sequential constrained decision problem. For campaign state s and eligible action a, the ideal objective is to maximize expected final externally verified task utility subject to every resource cap, trusted verdict rules and the frozen protocol. Estimating all possible future states is expensive and execution outcomes are uncertain, so the first research policy uses a greedy one-step ranking.

For each cumulative and separately metered resource i, let B_i be its cap, U_i usage and V_i outstanding reservations. Remaining R_i is B_i minus U_i minus V_i. Compute available capacity with checked integers and reject an invalid ledger state. Require 0 ≤ estimated upper bound c_i(a) ≤ R_i. For EXPLORE, also satisfy the protected floor. Memory and concurrency ceilings are admission limits rather than cumulative charges. A hard zero remaining cap permits only zero demand on that dimension.

```text
normalized_cost(a) = max(epsilon, max_i [c_i(a) / R_i for R_i > 0])
score(a) = (estimated_gain(a) + lambda * uncertainty_reduction(a)
            + mu * claim_importance(a)) / normalized_cost(a)
```

Use epsilon when the inner set is empty or every fraction is zero. Record estimates on compatible fixed scales before ranking. A pilot estimate may use task priors and completed run observations. A candidate with no dependency information receives direct claim importance. A proposed graph-aware estimate adds frozen weights of distinct reachable unresolved dependents, capped at the declared scale, using no hidden evaluation information. Proposed replay uncertainty is the decrease from resolving a required evidence slot. Pilot calibration must define the units, weights, cap and source of all estimates before the held-out freeze.

Compare scores exactly. The native scheduler uses unsigned 16-bit feature values and weights, a 64-bit vector for cumulative resources and epsilon of 1/1,000,000. The weighted sum is less than 2³³, so multiplying by a single cost denominator fits in 128-bit arithmetic. Compare the resulting ratios using integer quotients and reciprocal remainders so the comparison itself needs no wider product. For exact ties, use ascending stable action ID, with IDs assigned by the controller. The estimator can use a stable ordinal heuristic pending pilot calibration.

For A candidates and D metered dimensions, scanning precomputed summaries costs O(A × D) with bounded-width arithmetic. The native scheduler first sorts candidate IDs for a stable trace and duplicate detection, adding O(A log A) ID comparisons and O(A) memory. If each graph-aware candidate walks N claims and E dependency edges independently, add O(A × (N + E)) time and O(N) visited state per traversal. Profile before adding indexes or cached descendant summaries, since invalidation makes cache correctness consequential.

Failure modes include preference for cheap but redundant checks, starvation of an expensive prerequisite, overconfidence in correlated replay, underestimated cost and overweighting high fan-out claims. The hard reserve bounds some exploration starvation. Log delays, denied actions and aborted experiments to make these failures observable. Compare a fixed allowance, a cost-aware flat policy and a feasible random-development policy. Later candidates include contextual bandits for calibrated repeated tasks, Bayesian value of information when priors are defensible, and constrained planning when lookahead cost is affordable. Each would change the experimental policy and needs a new frozen comparison.

### Worked example

At 1000 remaining time units and 200 token units, a replay estimated at (100, 20) with numerator 0.4 has cost 0.1 and score 4. An exploration estimated at (100, 120) with numerator 0.9 has cost 0.6 and score 1.5. If the protected remaining token floor were 100, exploration would be filtered before scoring. This example illustrates arithmetic with hypothetical units.
