# AETHER continuation

Start with [project status](PROJECT_STATUS.md), [test results](docs/testing/TEST_RESULTS.md), and the [local launch guide](docs/operations/LOCAL_WORKBENCH.md). The owner authorized implementation-first development. The approval boundaries in [AGENTS.md](AGENTS.md) remain in force.

| Order | Work | Gate |
|---|---|---|
| 1 | Extend the reviewed task registry with a second separately frozen problem and compare alternative variants with declared selection and cost measures | Require appropriate task rights, fixed seeds and common verification |
| 2 | Qualify process-tree isolation, OS-enforced CPU and memory ceilings, no egress and credential isolation on a supported profile | Separate approval before executing untrusted or externally reviewed workers |
| 3 | Qualify an already-installed offline local model with no cloud features. Add other literature connectors as their service requirements and licensing permit | Explicit provider setup and external-transfer authorization where applicable |
| 4 | Validate GPU workload limits, remote authentication and durable queue recovery on approved environments | Optional infrastructure only. No new resources, broad privileges or deployment without approval |
| 5 | Refine A–E study design, run a pilot, freeze held-out cases and evaluate with independent review | No tuning against held-out data. Record null and negative findings |

The Windows launch, SSE resume, idle-stream Ctrl-C, active-worker cancellation, abrupt supervisor recovery and backup/restore drills passed in the [validation record](docs/testing/TEST_RESULTS.md). Use `aether-lab recover <id>` after confirming an interrupted supervisor has exited. Live supervisors and unexpired attempt leases block reconciliation. Use a fresh campaign ID for a new run and preserve the prior report. [Security review](docs/security/SECURITY_REVIEW.md)
