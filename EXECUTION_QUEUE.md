# AETHER execution queue

| Work | State | Release condition |
|---|---|---|
| DAE-01 reviewed knapsack discovery | Complete | Bounded seeded search, independent candidate re-score, exact oracle and SHA-256 lineage report verified locally. [Checkpoint](RESUME.md#dae-01-constrained-discovery) |
| DAE-02/03 development comparison | Complete | Matched random, density-greedy and evolutionary searches with separate cost measurements and fresh-process replay. [Results and scope](RESUME.md#dae-0203-paired-development-benchmark) |
| DAE-04 adaptive hybrid | Complete | Replayed the development-only ablation grid and the original baseline export. [Measured results and limitations](RESUME.md#dae-04-adaptive-hybrid-discovery) |
| DAE-05 reliability-aware ECAR | Complete | Replayed six policy arms on the development grid and confirmed original baseline parity. [Ablation results](RESUME.md#dae-05-reliability-aware-ecar-development-experiment) |
| DAE-05B reliability ablation | Frozen | `170d354` pins the predefined sensitivity grid and its results. [Results](RESUME.md#dae-05b-reliability-and-information-bonus-ablation) |
| Reliability follow-up | Future research | Test the information bonus on independently selected development families with partially dependent receipt sources. |
| DAE-06 adaptive inference | Complete | Compared four bounded strategies with deterministic fixtures and a qualified local Qwen pilot. [Results](RESUME.md#dae-06-adaptive-test-time-compute) |
| Adaptive inference follow-up | Proposed | Predefine a larger development grid and rotate live strategy order to account for cold model loading. |
| DAE-07 native performance laboratory | Complete | Reproducible release suite records randomized-order raw samples, robust summaries, uncertainty, workload boundaries and host telemetry. No optimization was made without a profiler-supported hotspot. [Results](RESUME.md#dae-07-native-performance-laboratory) |
| DAE-08 AETHER Research OS | Complete | Seven responsive research command pages, preserved existing workflows, authenticated read-only archive routes and browser-verified SHA-256 raw downloads. Desktop/mobile smoke and affected API/frontend gates passed. [Implementation and limitations](RESUME.md#dae-08-aether-research-os) |
| Discovery follow-up | Proposed | Evaluate seeded local search on separately selected development instances with explicit operator work or elapsed-cost caps and repeated timing. |
| ECAR held-out study | Deferred | Independent custody, frozen manifest and separate owner and reviewer approval. [Release procedure](docs/research/ATH_02B_ECAR_PROTOCOL.md#independent-custody-and-release) |
