# AETHER continuation

Start with [project status](PROJECT_STATUS.md), [test results](docs/testing/TEST_RESULTS.md), and the [local launch guide](docs/operations/LOCAL_WORKBENCH.md). The owner authorized implementation-first development. The approval boundaries in [AGENTS.md](AGENTS.md) remain in force.

| Order | Work | Gate |
|---|---|---|
| 1 | Review and explicitly approve the frozen `athena-local-01` protocol. Run it with the hash-pinned executable and retain the complete report. | The real local proposal and selected citation are frozen. Approval and execution are pending. See [RESUME.md](RESUME.md). |
| 2 | Get independent review of the held-out ECAR protocol, pin the implementation, and arrange independent task and label custody for an executable main study. | Development outcomes and analysis are reproducible. The frozen synthetic held-out specification is unexecuted. [Diagnostic](docs/research/ATHENA_ECAR_PROTOCOL.md) |
| 3 | Qualify OS memory and CPU ceilings, restricted identity, filesystem boundaries and egress controls. | Windows Job Objects contain the reviewed process lifecycle. Externally supplied work remains outside the registry. [Containment](docs/security/ATHENA_WORKER_CONTAINMENT.md) |
| 4 | Compare equivalent inference backends or quantizations and measure sustained concurrent local throughput if those constraints become material. | Current measurements cover one installed GPU model, one host and sequential routes. [Inference](docs/operations/ATHENA_INFERENCE.md) · [Benchmarks](docs/research/BENCHMARK_RESULTS.md) |

The Windows launch, SSE resume, idle-stream Ctrl-C, active-worker cancellation, abrupt supervisor recovery and backup/restore drills passed in the [validation record](docs/testing/TEST_RESULTS.md). Use `aether-lab recover <id>` after confirming an interrupted supervisor has exited. Live supervisors and unexpired attempt leases block reconciliation. Use a fresh campaign ID for a new run and preserve the prior report. [Security review](docs/security/SECURITY_REVIEW.md)

The [audit backlog](docs/engineering/PROMETHEUS_AUDIT.md) records Prometheus remedies and acceptance checks. The [release measurements](docs/research/BENCHMARK_RESULTS.md) record the optimization and evolutionary development runs. The [Athena reproducibility manifest](research/publications/ATHENA_REPRODUCIBILITY.md) pins the development, model and timing evidence.
