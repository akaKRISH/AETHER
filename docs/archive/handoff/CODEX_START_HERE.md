# Codex bootstrap — AETHER

Act as a principal Rust systems engineer, AI research scientist, security architect and beginner-friendly mentor. Create a fresh local workspace at `%USERPROFILE%\PROJECTS\AETHER` (resolve home dynamically). Preserve existing work. Read all current-v0.2 files and keep unchanged source copies under docs/archive/v0.2. v0.2 supersedes v0.1.

## Mission and owner decisions
AETHER is the Autonomous Research and Engineering Laboratory: a human-governed system for hypothesis generation, computational experiments, independent verification, reproducibility and algorithm discovery. Goals: portfolio, research publication, open-source ecosystem and possible startup. Maximize original engineering over a year or longer; teach the second-year B.Tech CSE owner from scratch. Hardware: Windows laptop, i5-13420H, RTX 3050 6 GB; optional free cloud GPUs. MVP runs CPU-only without paid API or LLM.

## Design authority
Rust Edition 2024 is the trusted control plane: ECAR scheduler, evidence graph, state, budgets, verification, persistence, runner and CLI. Use Cargo workspaces; Python only for isolated scientific workers through versioned contracts. SQLite initially, Axum/Tokio and UI later if justified. Core deterministic and independent of I/O. ECAR (Evidence-Constrained Adaptive Research) investigates budget-aware selection between exploration, replay, challenge and stopping using claim dependencies and evidence. Originality and efficacy must be tested rather than asserted. Trusted evaluators, not LLMs or workers, determine verdicts. Preserve audit history, provenance and negative results. Compare against fixed and adaptive baselines under equal budgets, including graph and scheduler ablations.

## Initial tasks
1. Read source specifications and diagrams. Inventory environment without modifying settings.
2. Create README.md, AGENTS.md, PROJECT_CONTEXT.md, PROJECT_STATUS.md, docs/index.md and docs/AETHER_MASTER_BLUEPRINT.md; archive sources unchanged.
3. Organize canonical PRD, TRD/SRS, HLD/LLD, stack, data/API contracts, security, research methodology, benchmark, SDLC/RDLC, QA, DevOps, risks, ADRs, UX/applications, references, learning and career alignment. Resolve contradictions explicitly, use traceable requirements and acceptance criteria. Review primary sources when available; never invent citations or results.
4. Set up directories crates/{aether-core,aether-engine,aether-store,aether-runner,aether-cli,aether-api}, workers/python, contracts, taskpacks, research, benchmarks, tests, infra, scripts, web and .github. Documentation-only placeholders are sufficient; no unimplemented active Cargo members.
5. Document a deterministic classification fixture CLI as the first milestone: input validation, metrics, ownership, structs, enums, Result and tests. No application implementation until owner approves.
6. Security: default-deny permissions, reviewed workers only, secret isolation, bounded resources, no arbitrary generated commands or network by default; Rust does not replace sandboxing.
7. Validate links, JSON, diagrams where tooling allows, requirements traceability, source integrity and repository structure. Initialize local Git if needed; no remote/push/purchases without explicit approval. Report exact path, inventory, decisions, blockers, actual validation results and next learning lesson.

Before each major implementation milestone explain concepts, alternatives, tradeoffs, exercises and quiz the owner. No fabricated research, benchmarks, tests or claims of global novelty.
