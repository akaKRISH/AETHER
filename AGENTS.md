# AETHER agent instructions

Read [PROJECT_CONTEXT.md](PROJECT_CONTEXT.md), [PROJECT_STATUS.md](PROJECT_STATUS.md), the [master blueprint](docs/AETHER_MASTER_BLUEPRINT.md), and the relevant archived v0.2 chapter before modifying the project. Check the current phase and approval record each session.

## Authority and change control

1. The project owner's approved decisions and the original Rust-first v0.2 design govern scope. Treat later owner-approved amendments as explicit, numbered decision records.
2. Use the canonical Genesis documents for implementation detail and traceability. Escalate unresolved conflicts in [SOURCE_RECONCILIATION.md](docs/SOURCE_RECONCILIATION.md). Keep the archive byte-identical.
3. The Python-first v0.1 documents are historical. Keep implementation proposals and scientific hypotheses clearly marked.
4. Update context, status, the relevant requirement/test links, and decision records with each approved material change.

## Engineering and research

- Rust Edition 2024 owns domain transitions, evaluator and verdicts, policy, budgets, orchestration, repository state, persistence, runner and CLI. Keep core deterministic and free of I/O. Python is restricted to reviewed scientific work through explicit contracts.
- Prefer the smallest correct change, native platform features and existing helpers. For code discovery in an indexed repository, prefer codebase-memory-mcp. Use text search for docs/configs and when the graph is unavailable. Discover Ruflo for complex multi-file work when available. Use local Memanto for relevant prior decisions if configured. Keep durable AETHER decisions in this repository.
- Only trusted evaluation plus applicable independent evidence can establish VERIFIED under a frozen protocol. Preserve failures, contradictory evidence and historical verdicts. Distinguish specifications, hypothetical examples and measured results. Cite primary research and constrain originality claims.
- Deny by default. Run reviewed benign workloads under supported isolation when workers arrive. Keep hidden labels, credentials and authority outside worker context. Constrain output and resources, validate all external input, and bind execution approval to a reviewed protocol and policy. See the [threat model](docs/security/THREAT_MODEL.md).
- Add a runnable check for non-trivial behavior. Report exactly what ran. Use the [test plan](docs/testing/TEST_PLAN.md) and [quality gates](docs/testing/QUALITY_GATES.md).

## Human approval and teaching

Follow the owner-authorized implementation-first policy. Build, validate, reverse-engineer, teach, then innovate. Proceed through eligible milestones in dependency order and record the actual results and continuation. Explain the important decisions, tradeoffs, source code and exercises alongside each implemented component. Student submissions are optional for implementation progress.

Retain separate approval for destructive operations, paid resources, public deployment, external transfers, broader execution permissions and major architecture changes. See [project status](PROJECT_STATUS.md) and [human approval](docs/governance/HUMAN_APPROVAL_POLICY.md).

Use project-relative paths in repository artifacts and descriptive linked requirement IDs. Keep engineering and research outputs small and reproducible. Where development instructions conflict, follow the explicit owner decision and record it.
