# AETHER — Claude + Codex + Antigravity Multi-Agent Quickstart

**Purpose:** Safely coordinate three agents on one AETHER milestone without merge chaos, duplicated work, hidden permission changes, or unsupported scientific claims.

## Recommended operating model

- **Claude:** architecture, research-methodology design, threat modelling, independent review. Read-only by default.
- **Codex:** Rust/backend implementation, schemas, persistence, CLI/API, deterministic tests.
- **Antigravity:** React/TypeScript, Research OS UI, responsive behavior, browser tests, documentation/visual QA.
- **Human owner:** scope, approvals, base commit, task ownership, integration, final commit.

Do not let two agents edit the same working tree. Use separate Git worktrees and branches, and agree on shared API/schema contracts before implementation.

## Current handoff baseline

- Repo: `C:\Users\2k25c\PROJECTS\AETHER`
- Latest reported commit: `d6e61bdc477fbff3ee2c5f737b4759a30fb81503` (APO-10)
- Latest reported worktree: clean
- APO-10 evaluation report: `research/results/apo-10-workflow-contract-evaluation.json`
- APO-10 reported results: 9/9 workflow fixtures; 7/7 adversarial checks; worker-report and receipt digests matched across two fresh processes; 138 workspace tests; 9 containment tests ignored by default, with harness checks run; 136 frozen research/archive Git blob identities preserved.
- Scientific-quality metrics unavailable; evidence still unverified. No held-out study or training ran.
- Verify all of this in the actual checkout before work starts.

## First steps for the human owner

```powershell
Set-Location 'C:\Users\2k25c\PROJECTS\AETHER'
git status --short
git branch --show-current
git log -1 --oneline
git worktree list
```

Check `AGENTS.md`, `RESUME.md`, `PROJECT_STATUS.md`, `EXECUTION_QUEUE.md`, relevant ADRs, and the APO-10 report. Do not assume the handoff is newer than the repository.

## Create isolated worktrees

Only do this after confirming the repository is clean and the desired base commit is correct. These are example names; use unique names for each milestone.

```powershell
Set-Location 'C:\Users\2k25c\PROJECTS\AETHER'
git worktree add -b agent/apo11-codex ..\AETHER-apo11-codex HEAD
git worktree add -b agent/apo11-antigravity ..\AETHER-apo11-antigravity HEAD
git worktree list
```

Claude should usually review read-only or use its own separate worktree. Do not share one mutable checkout between agents.

## Suggested APO-11 division of work

| Workstream | Owner | Deliverable |
|---|---|---|
| Evaluation protocol, scientific constructs, reviewer rubric, independence safeguards | Claude | Design proposal, threat-to-validity register, acceptance criteria; no shared code edits |
| Rust evaluator types, persistence, canonical reports, API and tests | Codex | Backend implementation in its own worktree |
| Research OS reviewer workflow, report inspection UI, responsive/browser tests | Antigravity | Frontend implementation against a frozen API contract in its own worktree |
| Review and integration | Human owner + Claude review | Serial cherry-pick/merge, final full test gates, docs and local commit |

Do not start implementation until the owner approves the protocol and the backend/frontend contract. If the work cannot be cleanly partitioned, let one agent implement and have the others review/test-design instead of parallel editing.

## Global boundaries — paste into every agent

```text
AETHER is a Rust-first Personal Intelligence OS and Scientific Discovery Partner.
The live repository and its AGENTS.md/current status documents outrank this
handoff. Verify branch, HEAD, worktree state, and relevant files first.

Never invent results, promote exploratory work to confirmatory evidence, treat
shape-valid evidence as verified, or equate passing software tests with
scientific truth. Preserve null findings and uncertainty.

Use default-deny capabilities, exact immutable workflow/skill approval binding,
and only reviewed Rust-native operations through the APO-09 kernel. No arbitrary
shell/Python/JavaScript execution through skills, downloaded code, dynamic plugin
binaries, unrestricted network/filesystem access, unapproved model calls, or
training. Do not claim hard cancellation, OS isolation, or cryptographic approval.

Do not modify frozen research/archive artifacts without explicit owner approval.
Do not access held-out data without an approved independent custody protocol.
Do not push, publish, deploy, install heavy dependencies, make system-level
changes, or perform destructive cleanup without explicit approval.

Work only in your assigned worktree and file scope. Do not edit another agent's
files. Agree on shared contracts before changing them. Prefer narrow diffs,
idiomatic Rust, targeted tests, and short continuation prompts. Report exact
commit, changed files, tests run/skipped/ignored, failures and limitations.
```

## Integration procedure

1. Each agent commits on its own branch and reports its commit hash.
2. Review diffs and ensure changed files match the assigned ownership.
3. Integrate one branch at a time into the designated integration branch.
4. Resolve shared contract changes explicitly; do not silently accept API drift.
5. Run full Rust, platform, Clippy, formatting, release, frontend, browser-smoke, and documentation gates as applicable.
6. Re-check frozen research/archive Git blob identities.
7. Update `RESUME.md`, `PROJECT_STATUS.md`, and `EXECUTION_QUEUE.md`.
8. Ask Claude for an independent review of the integrated diff and methodology.
9. The human owner authorizes the final local commit. Push/deploy only with explicit approval.

Avoid `git reset --hard`, `git clean -fdx`, and broad deletion for routine cleanup. Confirm the exact target before removing a worktree or temporary fixture directory.

## Suggested next milestone

**APO-11 — Independent Research Quality Evaluation** is a proposal, not yet an approved implementation. It should separately assess literature coverage, claim-evidence support, counterevidence, hypothesis falsifiability, experiment design, uncertainty, and abstention. It must separate software contract correctness from scientific quality and use independent human/oracle review where appropriate. Do not create one unsupported overall intelligence score.

A useful sequence is:

1. Claude designs the evaluation protocol and threat-to-validity analysis.
2. Human owner approves scope, metrics, and acceptance criteria.
3. Codex implements the evaluator and evidence/report persistence.
4. Antigravity implements the inspection/reviewer UI against the agreed API.
5. Serial integration, full validation, independent review, local commit.

## Model and prompt efficiency

Every coding task prompt should specify an available model, reasoning level, and **Standard** speed, plus a fallback. Use Medium reasoning for bounded implementation; High for security-sensitive architecture, threat modelling, or research-methodology design. Do not assume a particular model, subscription, quota, or feature is available. Avoid repeatedly loading the full project history; direct agents to targeted files and concise continuation notes.
