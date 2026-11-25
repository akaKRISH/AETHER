# 02 | Competitive research landscape and contribution

## What already exists
| System / work | Established capability | Implication for AETHER |
|---|---|---|
| AlphaEvolve [1,2] | Evolutionary coding with executable scoring; deployed optimization examples | Candidate generation and scoring are not novel. |
| AI Scientist v2 [3] | End-to-end automated research and workshop-reviewed output | Generating a manuscript is not the differentiator. |
| PaperBench [4] | Rubric-based assessment of paper replication | Replication scoring already exists; adopt comparable rigor. |
| MLE-bench [5] | ML engineering competition tasks | Accuracy alone cannot establish scientific reliability. |
| ScienceAgentBench [6] | Data-driven scientific coding evaluation | Start with bounded executable tasks. |
| FIRE-Bench [7] | Rediscovery of scientific insights | Rediscovery itself is not new. |
| SciAgentArena [8] | Real-world scientific challenges across scales | Do not claim first general-purpose evaluation. |
| Anthropic harnesses [9,10] | Durable session state, tools and long-running work | Durable orchestration is engineering groundwork. |
| Verification-gap survey [11] | Recent preprint highlights weak claim-verification artifacts | Motivates investigation, not proof of novelty. |
| Autonomous Science Bench [12] | Recent preprint studies decisions under experiment budgets | Budgeting alone is not a defensible novelty. |

## Candidate contribution: Evidence-Constrained Adaptive Research (ECAR)
Define a typed claim graph whose edges point to machine-verifiable artifacts, independent replay and disconfirming results. A budget-aware policy decides whether to explore, replicate, challenge or abstain. It cannot grant VERIFIED status itself; a separate deterministic verifier evaluates preregistered rules. Distinctive research claim to test: coupling explicit claim dependencies to verification allocation yields more verified utility per unit budget than a matched agent with fixed or heuristic verification. This is a proposed contribution, not established originality.

## Competing approaches and ablations
Single-agent execution is the cheapest baseline; fixed verification allocates the same budget fraction to replay; evolutionary search explores alternatives using an objective; multi-agent critique may improve reasoning but can consume budget without external evidence. AETHER's policy competes against these under identical model, tools, task packs and budgets. Ablate evidence-aware scheduling and graph structure separately; log both wall-clock and model cost. If graph overhead does not improve outcomes, retain the simpler architecture. If independent replay merely repeats a faulty metric, add hidden evaluator and challenge cases rather than claiming correctness.

## Novelty gate before research submission
Run a documented systematic search of arXiv, Semantic Scholar, OpenReview and recent conference proceedings using terms claim provenance, verification-aware agents, budgeted experiment selection, evidence graphs and scientific rediscovery. Record inclusion criteria, dates, related-work table and closest methods. Search patents only if startup IP is relevant. Obtain faculty review; revise the research question if a substantially equivalent method exists. AETHER's code, task suite, empirical ablations and negative findings can still be useful without a first-of-kind claim.
