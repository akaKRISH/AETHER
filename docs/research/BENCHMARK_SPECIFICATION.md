# AETHER-Mini benchmark specification

**Status** — Proposed benchmark. The first native fixture is a smoke test for exact metric and integrity behavior.

| Task family | Development example | Held-out question | Trusted oracle |
| --- | --- | --- | --- |
| Classification | Small reviewed digits or tabular task | Is a frozen candidate comparable to its baseline on a sealed split? | Alignment and label-domain checks, accuracy and optional calibration with frozen rules |
| Algorithm optimization | Sorting or numerical kernel on synthetic inputs | Does the candidate preserve semantics and improve timing under controlled inputs? | Reference function, declared numeric tolerance, varied input shapes, repeat timing after warmup |
| Reproduction | Small published ML finding with an accessible artifact | Can fresh code or a rerun recover the declared effect within tolerance? | Recomputed metric, effect direction, frozen threshold and required replay |

A task-pack manifest records a stable family and version, public development inputs, fixture expectations, data lineage and licenses, baseline reference, scoring direction, acceptance range and tolerance, artifact roles and output limits, environment policy, resource caps, replay checks and independently controlled hidden evaluator references. The trusted grader binds an attempt to the expected split and dataset digest before computing a score. Candidate-facing artifacts exclude held-out labels and scorer-write access. Keep split membership and release notes versioned.

Before freeze, run a valid baseline, known wrong prediction fixture, empty and duplicate IDs, label leakage guard, malformed manifest, verdict invalidation fixture and fresh rerun. Of these, exact metric and input validation apply immediately to the small native milestone. Process isolation, independently checked leakage and broader challenge fixtures apply after the respective execution and evidence features exist.

For timing, fix CPU/GPU and thread settings and record warmup and input shape. When CPU-only, GPU utilization is inapplicable. If hardware drift changes the noise band, split the comparison by hardware class or pause the study under its registered exception rules.

An immutable dev task set supports tuning. Separate held-out storage maintains assessment independence. Document potential public dataset memorization and curator leakage. Freeze package/evaluator digests before inspecting results and report all approved deviations. See [evaluation metrics](EVALUATION_METRICS.md).
