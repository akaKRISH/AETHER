# Development benchmark results

## Athena development evidence

Two fresh release invocations reproduced all 1,440 synthetic outcomes. Graph weighting relative to adaptive flat scheduling averaged +0.17 normalized points (95% seed-block bootstrap interval −0.84 to +1.19). Valuable hubs improved and unreliable hubs lost utility. A formal held-out run remains pending. [Protocol and complete arm results](ATHENA_ECAR_PROTOCOL.md) · [Raw data](../../research/results/athena-ecar-pilot.json)

The local RTX 3050 ran Qwen 3.5 4B Q4_K_M at confirmed 2,048 and 4,096 resident context lengths. Cold first text was 16.02 s and 16.49 s respectively. Warm medians were 232 ms and 224 ms. The model drafted a reviewed sorting proposal recorded separately from these infrastructure benchmarks. [GPU results](../operations/ATHENA_INFERENCE.md) · [Frozen draft](../../research/results/athena-local-draft-report.json)

The release report now shares the store connection across the campaign snapshot and digest-checked artifact reads. Each Windows loopback sample includes the full body. Both binaries served a fresh 36,973-byte fixture report. Each route had ten warmups and 50 sequential measured requests.

| Route | Before p50 | Before p95 | After p50 | After p95 |
|---|---:|---:|---:|---:|
| Health | 14.66 ms | 17.81 ms | 15.39 ms | 16.32 ms |
| Campaign snapshot | 14.43 ms | 24.07 ms | 15.64 ms | 16.58 ms |
| Events | 15.22 ms | 18.50 ms | 15.52 ms | 16.99 ms |
| Campaign report | 28.48 ms | 39.62 ms | 15.54 ms | 17.24 ms |

The server's peak working set was 13,193,216 bytes before and 13,201,408 after. These successive runs used different release binaries on one host. Windows scheduling and background load also vary. The measurements describe this run, and sustained concurrent campaign throughput requires a different measurement. [Before samples](../../research/results/athena-loopback-before.json) · [After samples](../../research/results/athena-loopback-after.json)

The final release repeated the 50-sample workload. Report p50 was 15.46 ms and p95 was 16.64 ms. The fresh report body was 36,961 bytes and the server peak working set was 13,201,408 bytes. [Final recheck](../../research/results/athena-loopback-recheck.json)

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

## Prometheus development runs

### Maximum subarray and objective workflow

The maximum-subarray release campaign froze 128 signed integers per seed. The trusted verifier enumerated every nonempty interval. Kadane's candidate matched the exact maximum on both inputs. The largest-single-item claim was rejected with two retained counterexamples. The worker charges totaled 132 ms for the three attempts, with zero model tokens and zero paid cost. Elapsed time includes both methods and process overhead. [Campaign export](../../research/results/prometheus-max-subarray.json)

| Frozen input | Exact maximum | Kadane | Largest single item |
|---|---:|---:|---:|
| Seed 19, Explore and Replay | 623 | 623 | 127 |
| Seed 281, Challenge | 1,131 | 1,131 | 127 |

Replay repeats seed 19 in a fresh process. Challenge uses a distinct frozen input. All observations share the same evaluator and generated input distribution. The objective demonstration used the deterministic provider to produce a draft and kept approval separate. A fresh process exported the same completed report. [Draft and planner provenance](../../research/results/prometheus-objective-draft.json) · [Completed objective campaign](../../research/results/prometheus-objective-report.json)

The continuation repeated the workflow under protocol v3 with the provider advice included in the approved hash. An identical draft request reused the advice, a start before approval failed, and the completed run matched the fresh-process report byte for byte. This run used the offline provider. [Bound draft](../../research/results/prometheus-objective-bound-draft.json) · [Bound report](../../research/results/prometheus-objective-bound-report.json)

### Knapsack candidate search

`aether-lab evolve <seed> 48` compared four methods at equal evaluation-slot caps on the same 12 items within each seed. The independent exhaustive oracle computed the optimum for each fixture. Random search had the lowest gap in two fixtures and tied on the third. Every method missed the optimum. The experiment covers three development fixtures, with one search trajectory per arm per seed. [Protocol](KNAPSACK_DEVELOPMENT_STUDY.md) · [Raw archives](../../research/results/prometheus-knapsack.json)

| Seed | Optimum | Random gap | Evolution gap | Fixed verification gap | ECAR-guided gap |
|---|---:|---:|---:|---:|---:|
| 19 | 333 | 12 | 143 | 171 | 150 |
| 281 | 370 | 81 | 81 | 81 | 81 |
| 919 | 373 | 43 | 76 | 76 | 76 |

The separate 180-case matched policy simulation found no verified-utility change from graph weighting in its tested pairs. It retains the action-order differences and the adaptive versus fixed allocation contrast. [Prometheus policy pilot](PROMETHEUS_POLICY_PILOT.md)

### Native performance laboratory

Criterion 0.8.2 used the Rust 1.96.1 optimized bench profile on the Windows i5-13420H host. Each operation had one second of warmup and 20 samples. The durable append exceeded its two-second target and collected all 20 samples in approximately 2.57 seconds. The table reports the estimated time per operation and Criterion's 95% interval. [Scheduler harness](../../crates/aether-engine/benches/scheduler.rs) · [Evidence harness](../../crates/aether-store/benches/evidence.rs) · [Raw samples and estimates](../../research/results/prometheus-criterion.json)

| Operation | Estimate | 95% interval |
|---|---:|---:|
| Scheduler with trace, 3 actions | 1.205 µs | 1.171–1.245 µs |
| Scheduler with trace, 32 actions | 15.696 µs | 14.360–16.799 µs |
| Scheduler with trace, 128 actions | 78.719 µs | 75.699–81.332 µs |
| Reject a cycle in a 64-claim chain | 113.99 µs | 111.28–116.57 µs |
| Combined in-memory 64-claim evidence and journal snapshot | 1.221 ms | 1.196–1.248 ms |
| Append one durable SQLite decision | 11.576 ms | 10.512–13.473 ms |

The journal append used one temporary on-disk database with rollback journaling and full synchronous writes. The graph measurements used an in-memory database. The recovery drill took 173.39 ms to execute the release CLI after the interrupted attempt's lease became eligible. This timing includes process startup and excludes the lease wait. No matched language-speed or concurrent throughput comparison has run. [Recovery and validation](../testing/TEST_RESULTS.md)

The final release API was measured over loopback on one completed sorting campaign. Each route had ten warmup requests and 50 measured requests. Times include HTTP round trip and full response body read. Peak server working set was 13,082,624 bytes (about 12.5 MiB) during the run. [Timing harness](../../tests/measure-loopback.cjs) · [Raw route samples](../../research/results/prometheus-loopback-performance.json)

| Route | Body bytes | p50 | p95 |
|---|---:|---:|---:|
| Health | 34 | 14.78 ms | 16.70 ms |
| Campaign snapshot | 20,031 | 14.89 ms | 19.01 ms |
| Campaign event page | 12,963 | 15.26 ms | 17.44 ms |

The final release sorting worker measured 48–59 ms across nine process-inclusive runs on seeds 11, 19 and 281. Every output matched all 128 positions. Startup, scheduling noise and output capture contribute to the timing. [Native worker measurements](../../research/results/prometheus-native-worker.json)
