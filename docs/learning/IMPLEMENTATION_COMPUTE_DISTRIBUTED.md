# CPU measurement, GPU computing and distributed leases

Wall time counts elapsed time, including waiting, startup and I/O. CPU time counts execution on processors. Memory and device capacity constrain the jobs you can run together. A GPU performs many operations in parallel but can lose on small jobs when moving data and launching work dominate.

```mermaid
flowchart LR
    C["Controller starts native worker"] --> P["Process startup"]
    P --> K["Sort reviewed input"]
    K --> R["Read and validate receipt"]
    C --> T["Elapsed timer"]
    R --> T
    F["Future qualified remote or GPU worker"] -. requires evidence .-> C
```

## Read the measurement and its limits

1. In [`benchmark_native`](../../crates/aether-api/src/main.rs), loop through fixed seeds and the three approved algorithms. Construct the same-sized task for each and request supervision.
2. In [`supervise_child`](../../crates/aether-runner/src/lib.rs), start an `Instant` immediately before spawning a worker. Worker startup, sorting, output capture and receipt parsing are included in the resulting duration. Comparing small sorts under this boundary primarily teaches how to choose a measurement.
3. Return to `benchmark_native`. Check correctness independently, then attach measured wall time, output sizes, environment and binary digest. Recording the result is distinct from estimating an effect across machines or a representative workload.
4. Compare [`runtime`](../../crates/aether-api/src/http.rs), which exposes local CPU/OS and worker admission state. The `benchmark` command times the synthetic scheduler wrapper and records its modeled resource units separately. GPU compute and authenticated remote execution require separate implementation and qualification.

## GPU fundamentals before an experiment

A kernel maps work across device threads. Speed depends on memory access, parallel work per launch and numeric precision. Estimate (4N) bytes for N FP32 values and (2N) for FP16 before adding intermediate buffers, model weights, activations, cached tensors and runtime overhead. A 1000 × 1000 FP32 matrix occupies about 4 MB before allocation overhead. A workload that fits capacity can still bottleneck on memory bandwidth or transfers. Measure end-to-end wall time and kernel time separately with explicit warmup. Compare CPU and GPU answers using frozen tolerances and enough repeated inputs to estimate uncertainty. See the [hardware design](../architecture/HARDWARE_ARCHITECTURE.md).

## Distributed systems fundamentals before a worker

A lease gives one attempt bounded time to report. A fencing token identifies which lease May commit. Read [`complete_attempt` and `recover_expired`](../../crates/aether-store/src/execution.rs). The local controller uses a lease token and deadline, rejects stale results and supports an identical completion retry. In a distributed setting, messages can arrive late, twice or never. A worker cannot decide whether sending a result made it durable. Fence an old lease in the authoritative store and make completion idempotent before adding transport. A remote worker would also need authentication, compatible evidence records, per-attempt policy and measured isolation. See the [scalability design](../architecture/SCALABILITY_DESIGN.md).

**Alternative.** A CPU implementation has fewer setup dependencies and is easy to repeat. Device work can pay off for larger batches after accounting for transfers. Remote work can raise throughput after durable admission and cancellation semantics are demonstrated across network partitions.

**Exercise.** Run the native development timing and annotate what its wall clock includes. Draw a timeline with process startup lasting 4 units and a sort lasting 1 unit. If a theoretical accelerator halves sort time but adds 2 units of transfer, determine which end-to-end path is faster and what measurement would reveal the reason.

**Interview check.** Why can greater throughput increase the latency of an individual small request? How do deadlines and worker fencing prevent two terminal results after a partition? What distinguishes an at-least-once attempt from exactly-once ledger settlement?

**Debugging challenge.** An apparent sorting improvement came from swapping a busy worker process for a warm long-running one. List the variables that make the two timings incomparable and design a repeated timing sample that holds setup constant.

**Rebuild unit.** Write a small CPU-only timing harness that discards warmup, computes a median and verifies results independently. Add a simulated queue with a clock and monotonically increasing fencing tokens. Inject late and twice-delivered completions and verify one terminal ledger record.
