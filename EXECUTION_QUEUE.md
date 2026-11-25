# AETHER execution queue

| Work | State | Release condition |
|---|---|---|
| DAE-01 reviewed knapsack discovery | Complete | Bounded seeded search, independent candidate re-score, exact oracle and SHA-256 lineage report verified locally. [Checkpoint](RESUME.md#dae-01-constrained-discovery) |
| DAE-02/03 development comparison | Complete | Matched random, density-greedy and evolutionary searches with separate cost measurements and fresh-process replay. [Results and scope](RESUME.md#dae-0203-paired-development-benchmark) |
| DAE-04 adaptive hybrid | Complete | Replayed the development-only ablation grid and the original baseline export. [Measured results and limitations](RESUME.md#dae-04-adaptive-hybrid-discovery) |
| DAE-05 reliability-aware ECAR | Complete | Replayed six policy arms on the development grid and confirmed original baseline parity. [Ablation results](RESUME.md#dae-05-reliability-aware-ecar-development-experiment) |
| Reliability follow-up | Proposed | Isolate the information bonus and use development cases with more usable receipts per claim. |
| Discovery follow-up | Proposed | Evaluate seeded local search on separately selected development instances with explicit operator work or elapsed-cost caps and repeated timing. |
| ECAR held-out study | Deferred | Independent custody, frozen manifest and separate owner and reviewer approval. [Release procedure](docs/research/ATH_02B_ECAR_PROTOCOL.md#independent-custody-and-release) |
