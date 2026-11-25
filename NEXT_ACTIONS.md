# AETHER continuation

Start with [project status](PROJECT_STATUS.md), [test results](docs/testing/TEST_RESULTS.md), and the [local launch guide](docs/operations/LOCAL_WORKBENCH.md). The owner authorized implementation-first development. The approval boundaries in [AGENTS.md](AGENTS.md) remain in force.

| Order | Work | Gate |
|---|---|---|
| 1 | Obtain independent review and task-label custody for a formal ECAR study, then freeze the implementation, analysis and held-out inputs before its separate approval gate. | The synthetic development study is reproducible. Its public held-out specification has not been independently curated or executed. [Diagnostic](docs/research/ATHENA_ECAR_PROTOCOL.md) |
| 2 | Keep the completed local model campaign as a bounded demonstration and improve scope only with a reviewed task and new protocol. | The pinned sorting objective completed exploration, independent replay and challenge with trusted verification. [Report](research/results/athena-local-campaign.json) |
| 3 | Qualify OS memory and CPU ceilings, restricted identity, filesystem boundaries and egress controls. | Windows Job Objects contain the reviewed process lifecycle. Externally supplied work remains outside the registry. [Containment](docs/security/ATHENA_WORKER_CONTAINMENT.md) |
| 4 | Compare equivalent inference backends or quantizations and measure sustained concurrent local throughput if those constraints become material. | Current measurements cover one installed GPU model, one host and sequential routes. [Inference](docs/operations/ATHENA_INFERENCE.md) · [Benchmarks](docs/research/BENCHMARK_RESULTS.md) |

The Windows launch, SSE resume, idle-stream Ctrl-C, active-worker cancellation, abrupt supervisor recovery and backup/restore drills passed in the [validation record](docs/testing/TEST_RESULTS.md). Use `aether-lab recover <id>` after confirming an interrupted supervisor has exited. Live supervisors and unexpired attempt leases block reconciliation. Use a fresh campaign ID for a new run and preserve the prior report. [Security review](docs/security/SECURITY_REVIEW.md)

The [audit backlog](docs/engineering/PROMETHEUS_AUDIT.md) records Prometheus remedies and acceptance checks. The [release measurements](docs/research/BENCHMARK_RESULTS.md) record the optimization and evolutionary development runs. The [Athena reproducibility manifest](research/publications/ATHENA_REPRODUCIBILITY.md) pins the development, model and timing evidence.
