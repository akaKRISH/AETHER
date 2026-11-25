# When does dependency weighting help a bounded verifier?

Local working manuscript. Evaluation is a synthetic development pilot with a separate model-guided engineering demonstration.

## Abstract

An auditable research scheduler allocates bounded exploration and verification work across dependent claims. In a 1,440-outcome development simulation, incremental downstream weighting shifted utility by +0.17 points on average versus adaptive flat scheduling, with a 95% seed-block percentile interval of −0.84 to +1.19. Task-level differences included both gains and losses. A separately owner-approved local-model proposal also completed a frozen 96-item sorting task with trusted reference agreement, fresh-process replay and a distinct challenge.

## Introduction

Research controllers need to select the next experiment and justify trustworthy conclusions within fixed compute limits. Dependency structure may identify useful prerequisite checks. Dependency weights can also exaggerate the payoff from checking a weak prerequisite. Athena studies these cases under controlled inputs.

## Related work

CAVEAT represents claims from heterogeneous traces in a shared dependency graph. The paper estimates verifier sensitivity and specificity with Beta parameters and models shared slips within verifier families. Its greedy allocator compares expected reduction in dependency-weighted uncertainty per cost across claim and verifier pairs. Benchmarks in the paper use multiple heterogeneous verifiers across reasoning and claim-verification domains. The v1 review manuscript withholds its implementation and configuration. We found no verified author release for the analyzed version. Reproduction would require these artifacts and calibration data. [CAVEAT v1](https://doi.org/10.21203/rs.3.rs-11035327/v1)

ECAR and CAVEAT overlap in dependency-sensitive, budgeted verification. This implementation studies the interaction of priorities with executable experiments, protected verification budget, counterexample attempts, crash recovery and report provenance. These are contribution candidates. The current diagnostic supplies neither CAVEAT's calibrated correlated verifier portfolio nor a faithful prior-art baseline.

## Research question

How does adding downstream claim mass to the priority score change verified utility at a fixed budget, and how does that change vary with topology and prerequisite reliability?

## Mathematical formulation

Each claim has a positive direct weight. A simulation labels a claim true only if its local keyed draw and every ancestor satisfy the hidden truth rule. Replay and challenge observations use distinct noise and failure draws. Utility sums the direct weight of truly verified claims, divided by total possible weight. Graph arms give each claim an importance equal to its own weight plus the weight of its descendants. The shared Rust scheduler ranks each eligible action against its normalized resource cost. [ECAR formulation](../../docs/research/ECAR_SPECIFICATION.md)

## System architecture

The local provider contributes reviewed candidate choices and a proposed hypothesis. Citation snapshots and model provenance enter the frozen protocol before approval. A Rust supervisor enforces admission and runs the reviewed task. The trusted evaluator determines the claim state. SQLite retains budgets, attempts and the evidence ledger. [Athena architecture decision](../../docs/decisions/ADR-013.md)

## Experimental methodology

We varied topology, observation costs, error rates and worker failure rates across five development scenarios. Three caps and 24 common seeds give 360 paired inputs. The four arms share hidden truth, costs, observations conditional on selecting an action, and the final verifier. Each seed is one inference block. Observed elapsed wall time for the scheduler and host run was recorded separately from simulated task charges.

## Evaluation protocol

The primary statistic is the seed-block mean paired difference in normalized verified utility between adaptive graph and adaptive flat. The analysis fixes 2,000 resamples, a common RNG seed, and a five-point planning effect. The predeclared minimum of 30 blocks governs the frozen held-out specification after the development sizing estimate. [Protocol and matching rules](../../docs/research/ATHENA_ECAR_PROTOCOL.md)

## Results

The main contrast had 62 wins, 238 ties and 60 losses. Across the development corpus, 3 of 582 ECAR verifications and 3 of 549 adaptive-flat verifications contradicted the separate simulated truth labels. The valuable hub benefited from graph weighting and the unreliable hub lost utility. The synthetic negative and null results remain in the raw export. [Outcomes](../results/athena-ecar-pilot.json) · [Analysis](../results/athena-ecar-summary.json)

## Ablations

Graph priority changed fixed-policy utility by −0.68 points (95% interval −1.39 to +0.06). Adaptive scheduling with flat evidence differed from fixed flat by +5.24 points (95% interval +4.52 to +5.98). The allocation policies differ in admissible verification spending, so the latter contrast includes this mechanism. A zero contrast under the flat topology provides a negative control for graph priority.

## Threats to validity

Scenario families are intentional fixtures. Common random numbers ensure a fair within-pair comparison. They leave outcomes correlated within each seed block. Noise and failure rates model one diagnostic regime. The source labels and scheduler share a designed corpus, so independent task custody and externally reviewed real evaluations remain necessary.

## Limitations

The formal held-out experiment is pending. The live model experiment uses one installed quantization and one bounded sorting task. All three verified exact-reference fractions were 1.0 for the frozen inputs. This is a single demonstration. The model is a proposer, while the deterministic verifier supplies task evidence. The model's other-algorithm speculation is outside the frozen task. These results cannot establish broad algorithmic superiority or research novelty.

## Reproduction

Use the [development configuration](../../benchmarks/athena-ecar-development.json), [diagnostic runner](../../crates/aether-engine/examples/ecar_diagnostics.rs), and [paired analysis](../../scripts/athena_ecar_summary.py). Match configuration hashes and compare outcome rows from two processes. Timings can differ with host load. The [held-out specification](../../benchmarks/athena-ecar-heldout-spec.json) remains frozen and unexecuted. See the [artifact manifest](ATHENA_REPRODUCIBILITY.md) for command paths and data hashes.

## References

- Guo et al., *CAVEAT: Reliability-aware active verification under correlated errors for multi-agent LLM reasoning*, v1. [Research Square](https://doi.org/10.21203/rs.3.rs-11035327/v1)
- C. A. R. Hoare, *Quicksort*, 1962, cited as historical background for the sorting task. [The Computer Journal](https://doi.org/10.1093/comjnl/5.1.10)
