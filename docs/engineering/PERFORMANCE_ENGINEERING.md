# Performance engineering

Measure workload before optimizing. The first fixture evaluates a length-n prediction vector in O(n) time with O(1) extra scalar state. Record its reproducible output before adding performance measurement. Worker startup, artifact ingest, evaluator compute and database transactions have separate timings to avoid attributing one bottleneck to another.

| Profile | Measurement |
|---|---|
| Fixture smoke | Exact metric, clean run and no external service |
| Artifact pathway | Bytes, hashing time, peak memory and duration by file size |
| Budget and scheduler | Allocation latency and time against actions and dependency edges |
| Runner | Startup, execution, cancel latency and peak resource usage |
| R2 API | Metadata-read p50/p95 and error rate at five clients, excluding worker/model latency |

Capture machine details, OS, build mode, dependency versions, power mode, sample count and warmup policy. Separate reruns for tuning from held-out results. Report medians and dispersion alongside raw observations when allowed. Freeze thresholds only after pilot calibration. Resource cap behavior remains a correctness check independent of performance results.

See the [resource budget](../architecture/RESOURCE_MANAGEMENT.md) and [source QA](../archive/v0.2/AETHER/15_TESTING_QA.md).
