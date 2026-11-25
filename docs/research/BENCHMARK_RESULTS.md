# Development benchmark results

AETHER provides two distinct checks. `aether-lab benchmark` executes matched policy arms on a deterministic synthetic observation model. `aether-lab benchmark-native` measures real process-inclusive wall time for reviewed sorting variants and verifies each against a reference. The synthetic study does not measure scientific performance and the native check is a small systems measurement.

See the [protocol](BENCHMARK_SPECIFICATION.md) and [originality assessment](ORIGINALITY_ASSESSMENT.md). Held-out evaluation and comparisons against close prior art remain open.

## Recorded run

The Windows x64 run used Rust and Cargo 1.96.1 on the local Intel Core i5-13420H host with 12 logical CPUs. The release `aether-lab.exe` SHA-256 was `c281b025c1615d04bde6f97fc9b2c09eaefd2bf00e0dc71d69ac71c97fdd92f5`. The campaign protocol and native timing export pin that executable. [Environment](../operations/DEVELOPMENT_ENVIRONMENT.md)

### Sorting campaign

`aether-lab demo local-sorting-01 19` froze and approved a bounded insertion-sort protocol. ECAR selected EXPLORE, REPLAY and CHALLENGE and then stopped. The candidate and replay used seed 19 in separate worker processes. The challenge used the frozen XOR-derived seed. The trusted verifier found all 128 output positions correct on both seeds. The sorting claim became `verified` under that protocol. The follow-up baseline comparison remained `proposed`. Across three attempts the worker wall-time charges totaled 138 ms, with zero model tokens and zero paid cost. The report includes the protocol, three manifests, artifact bytes, verdict, replay and decision traces. [Recorded campaign](../../research/results/local-sort-campaign.json)

### Native worker timings

`aether-lab benchmark-native` measured the following process-inclusive wall times in milliseconds. Each sample sorted 128 values and matched the reference in all 128 positions. Each cell contains one execution. Process startup and output capture are included in the measurements. [Raw timings](../../research/results/native-sort-benchmark.json)

| Seed | Standard | Insertion | Selection |
|---|---:|---:|---:|
| 11 | 44 | 45 | 49 |
| 19 | 50 | 51 | 51 |
| 281 | 47 | 49 | 50 |

### Matched synthetic simulation

`aether-lab benchmark` ran 45 simulated campaigns in 1,404 µs of total host wall time. The run crossed five policies A–E with seeds 11, 23 and 37 on flat, shared-prerequisite and challenged-prerequisite task structures. Utility and verification outcomes in this file come from the simulator's assumptions. The run exposes deterministic policy behavior under matched caps. Experimental ECAR efficacy and comparative algorithm performance remain open. [Raw simulation](../../research/results/ecar-policy-simulation.json)
