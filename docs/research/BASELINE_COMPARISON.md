# Baseline comparison

Use one Rust harness for all study configurations. Agents may propose tasks from the same allowlist and receive equivalent approved candidate-facing tools. Final evidence predicates and access to controller-permitted measurements stay constant across conditions. Record the exact scheduling inputs each policy can see.

| ID | Candidate strategy | Verification allocation | Scheduler access |
| --- | --- | --- | --- |
| A | One LLM proposal and trusted execution | No iterative generation or adaptive disposition. Shared external assessment still applies. | Frozen prompt and approved tool inputs |
| B | Iterative matched agent | Fixed preregistered replay allowance with deterministic selection | Flat claim checklist |
| C | Same iterative agent | Adaptive cost and uncertainty ranking | Flat claim checklist and same budget state |
| D | Same iterative agent | ECAR adaptive ranking across exploration, replay and challenge | Dependency graph and same budget state |
| E | Same iterative agent, optional | Fixed verification allocation with graph-aware selection within the allowance | Dependency graph and same budget state |

Freeze a fixed-policy rule with a deterministic schedule and the same exploration reserve as relevant adaptive policies, where feasible. Define how fixed B and E select exploration and how they select replay/challenge within their allowance. D and C share ranking components and feature scales apart from the graph contribution. Removing graph input from a scheduler leaves the trusted grader's graph/evidence requirements intact, so policy ablations cannot buy higher utility by lowering scientific requirements.

Report both hard cap parity and actual spend. A run ending early may spend less and still provide a fair matched-cap comparison. Budget-inclusive measures should include candidate inference, evaluation, failed attempts and optional independent retries when the protocol classifies them as in-campaign. Report actual unavailable model/GPU usage accurately. No external model is required for the earlier native fixture, which is outside this confirmatory comparison.

## Native synthetic simulation

The Rust engine's `ecar-synthetic-v1` protocol exercises all five policy paths on three small hand-authored fixture shapes. A takes one deterministic scripted proposal. B and E prioritize exploration among feasible candidates before spending from a fixed combined replay/challenge allowance. The graph-aware policies may prioritize a prerequisite's downstream value. All arms see identical hypothetical check outcomes for each task, seed, claim and check kind and share the same final fixture verdict rules. Resource costs are synthetic integers. Use these outputs to debug decisions, equality of conditions and budget accounting before designing the development pilot.
