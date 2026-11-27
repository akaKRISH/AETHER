# AETHER — Multi-Agent Shared Coordination Ledger (`AGENT_SHARED.md`)

> **Status:** Canonical Multi-Agent Coordination Ledger  
> **Designated Maintainer:** Google Antigravity (Gemini)  
> **Final Authority:** Project Owner  
> **Canonical Repository Path:** `C:\Users\2k25c\PROJECTS\AETHER\docs\agent-handoff\AGENT_SHARED.md`  
> **Last Updated:** 2026-11-27T12:11:00+05:30  
> **Active Base Commit:** `943ee72cce9dcaa0a0a06e76f75a7b4c229248f2` (SETUP-01 Checkpoint; APO-10 base `d6e61bdc477fbff3ee2c5f737b4759a30fb81503`)

---

## 1. Project Identity and Baseline

### 1.1 Project Vision and Objectives
AETHER is a Rust-first **Personal Intelligence OS** and **Scientific Discovery Partner** designed as a **Hybrid Intelligence Laboratory**. Its objective is to help formulate research questions, discover and structure scientific literature, construct versioned claim and evidence graphs, detect counterevidence and contradictions, design rigorous experiments, execute approved bounded workflows, independently evaluate outcomes, and maintain an immutable, reproducible audit record.

**Core Operating Principles:**
- **Reproducibility & Determinism:** Native Rust-first core, deterministic transitions, immutable content-addressed definitions and execution receipts.
- **Evidence Integrity:** Explicit separation of exploratory work from confirmatory evidence; provenance-backed claim dependencies; explicit uncertainty quantification; honest abstention when evidence is insufficient.
- **Security & Authorization:** Default-deny capabilities, exact immutable workflow and skill approval binding, reviewed native operations, no arbitrary code execution or unapproved external network/model calls.
- **Governance:** Independent evaluation rather than self-certification; strict change control governed by the Project Owner.

### 1.2 Current Repository State (Independently Inspected)
- **Local Repository Root (EXISTING):** `C:\Users\2k25c\PROJECTS\AETHER` (Canonical Integration Workspace)
- **Active Branch (EXISTING):** `main` (inspected 2026-11-27 via `git branch --list`)
- **Verified HEAD Commit (EXISTING):** `943ee72cce9dcaa0a0a06e76f75a7b4c229248f2` (SETUP-01 Checkpoint; APO-10 base `d6e61bdc477fbff3ee2c5f737b4759a30fb81503`)
- **Git Worktree Inventory (Actual State):**
  - `C:/Users/2k25c/PROJECTS/AETHER` — **EXISTING** (main working tree on branch `main`)
  - `C:/Users/2k25c/.local/share/kilo/worktree-pool/aether-1e8a3a9c6b47/inexpensive-petroleum` — **EXISTING** (detached background worktree at `d6e61bdc477fbff3ee2c5f737b4759a30fb81503`)
  - `..\AETHER-apo11-codex` — **PLANNED / NOT CREATED** (directory does not exist on disk; branch `work/apo11-codex` does not exist)
  - `..\AETHER-apo11-antigravity` — **PLANNED / NOT CREATED** (directory does not exist on disk; branch `work/apo11-antigravity` does not exist)
- **Working Tree State:** Clean working tree relative to Git index. Newly introduced files are untracked handoff and coordination documentation in `docs/agent-handoff/` and root briefing documents. Zero tracked files modified or staged.
- **Handoff Artifacts Inspected:**
  - `AETHER_FULL_CONTEXT_AND_PROGRESS_REPORT.md` (SHA-256: `EC58E4917AC7DCD9D6357F7F46D44915FED9B96FE65DECC081C244946719BA20`)
  - `AETHER_MULTI_AGENT_QUICKSTART.md` (SHA-256: `1017EAF007C2F42D7832B3C96A8B33101784CFF9C9CFDF8B4019078CB9CE5C95`)
  - `AETHER_AGENT_HANDOFF.zip`: **MISSING** (thoroughly scanned repository, user profile, Downloads, Desktop, and Documents; not present on filesystem).

### 1.3 Latest Reported Milestone Baseline (APO-10)
> *Notice: The following results and claims represent previously reported project history from handoff documentation and conversation records. They have not been independently re-executed or verified during this initial documentation setup task.*

- **Reported Commit:** `d6e61bdc477fbff3ee2c5f737b4759a30fb81503`
- **Reported Evaluation Report:** `research/results/apo-10-workflow-contract-evaluation.json`
- **Reported Test Outcomes:**
  - 9/9 workflow fixtures passed.
  - 7/7 adversarial checks passed.
  - 138 workspace unit and integration tests passed.
  - Two fresh processes produced matching report and receipt digests.
  - 136 frozen research and archive Git blob identities preserved.

### 1.4 Known Limitations and Unresolved Issues
The following limitations are explicitly noted in the project baseline and must not be misrepresented:
1. **Unverified Evidence:** Shape-valid evidence remains unverified; passing software tests do not constitute scientific truth.
2. **Missing Quality Metrics:** Scientific-quality evaluation metrics are unavailable; no held-out scientific study or training was executed.
3. **Ignored Containment Tests:** Nine containment tests remain ignored by default in standard test suites (harness-driven checks reported passed).
4. **Cooperative Cancellation:** Task cancellation is cooperative; no OS-level hard preemption or kernel sandbox is currently claimed.
5. **Temporary Fixture Artifacts:** One temporary fixture database directory remains under `%TEMP%`.
6. **SQLite Journal Adjustment:** The release smoke validation required a narrow SQLite journal-mode adjustment pending final review.

---

## 2. Agent Registry

| Agent | Primary Role | Assigned Responsibilities | Operating Boundaries & Restrictions | Platform / Model |
|---|---|---|---|---|
| **Claude** | Architecture & Research Advisor | Research methodology design, formal architecture, independent peer review, threat modelling, validity assessment. | **Web Chat Access Only.** Does not have local shell or filesystem access to the repository. Receives briefings via manual text/document upload. Must not assume local write access. | Claude AI Free (Web Interface) |
| **Codex** | Core Backend Engineer | Rust core, SQLite store, backend APIs, deterministic execution, CLI, and automated unit/integration tests. | Must operate strictly in dedicated worktree (`..\AETHER-apo11-codex`) on branch `work/apo11-codex` once authorized and created (**PLANNED / NOT CREATED**). Must not edit frontend, documentation handoffs, or shared schemas without prior contract agreement. | OpenAI Codex CLI / IDE Extension |
| **Antigravity** | Engineering Coordinator & Frontend Engineer | Documentation coordination, central ledger maintenance, React/TypeScript, Research OS UI, browser testing, and integration. | For UI code, operates in dedicated worktree (`..\AETHER-apo11-antigravity`) on branch `work/apo11-antigravity` once authorized and created (**PLANNED / NOT CREATED**). Designated routine maintainer of `AGENT_SHARED.md` and central coordination logs in canonical checkout. | Google Gemini through Antigravity |
| **Project Owner** | Final Authority & Integrator | Overall project direction, scope, milestone approvals, architecture sign-off, security decisions, and final integration. | Final decider on all code merges, architecture decisions, and policy overrides. | Human Owner |

*Rule:* Record only actual reported models; never fabricate model versions or capabilities.

---

## 3. Global Task Board

Allowed Task Statuses: `PROPOSED`, `READY`, `IN_PROGRESS`, `BLOCKED`, `IN_REVIEW`, `COMPLETED`, `CANCELLED`.  
*Rule:* Every task must have exactly one primary owner.

| Task ID | Description | Primary Owner | Status | Priority | Branch / Worktree | Owned Files / Directories | Dependencies | Agreed Interfaces | Reviewer | Start / Last Update | Deliverable | Completion Commit |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **HIST-APO07** | Declarative Skill Registry, Typed Schemas, Default-Deny Capabilities and Approval System | UNKNOWN — NOT YET VERIFIED (Committed by akaKRISH) | `COMPLETED` *(Reported)* | High | `main` | `crates/aether-api`, `web/`, `tests/`, `docs/decisions/ADR-021.md` | APO-06B (`5f30f71`) | ADR-021 (Skill Schemas) | UNKNOWN — NOT YET VERIFIED (local_operator self-receipt) | Start: UNKNOWN / Last: 2026-11-26T16:57:25+05:30 | Declarative skill registry & approval system | `b400904eecc18c4dadbeae37daeabcc31a57b338` |
| **HIST-APO09** | Bounded Skill Execution Kernel, Reviewed Rust Operations, Capability Checks and Content-Addressed Execution Receipts | UNKNOWN — NOT YET VERIFIED (Committed by akaKRISH) | `COMPLETED` *(Reported)* | High | `main` | `crates/aether-api`, `web/`, `tests/`, `docs/decisions/ADR-022.md`, `RESUME.md` | APO-07 (`b400904`) | ADR-022 (Kernel Checks) | UNKNOWN — NOT YET VERIFIED (local_operator self-receipt) | Start: UNKNOWN / Last: 2026-11-26T19:10:11+05:30 | Bounded skill execution kernel & receipts | `ef4f5bfcba83c9e801c3e757da830278680809a5` (checkpoint pin; code in `6c507e5b271c6baebf9558bcc00282f190ce942e`) |
| **HIST-APO08** | Deterministic Research Workflow Orchestrator, Content-Addressed Workflow DAGs, Sequential Execution and Replayable Receipts | UNKNOWN — NOT YET VERIFIED (Committed by akaKRISH) | `COMPLETED` *(Reported)* | High | `main` | `crates/aether-api`, `web/`, `tests/`, `docs/decisions/ADR-023.md` | APO-09 checkpoint (`ef4f5bf`) | ADR-023 (Workflow DAG Schemas) | UNKNOWN — NOT YET VERIFIED (local_operator self-receipt) | Start: UNKNOWN / Last: 2026-11-26T22:00:40+05:30 | Deterministic workflow orchestrator & DAG engine | `d20d0a755ded1cbcd2f19389620bc8b995399b54` |
| **HIST-APO10** | End-to-End Research Workflow Contract Evaluation | UNKNOWN — NOT YET VERIFIED (Committed by akaKRISH) | `COMPLETED` *(Reported)* | High | `main` | `crates/aether-api`, `crates/aether-store`, `web/`, `tests/`, `research/results/` | APO-08 (`d20d0a7`) | aether-workflow-contract v1.0.0 | UNKNOWN — NOT YET VERIFIED (local_operator self-receipt) | Start: UNKNOWN / Last: 2026-11-27T10:12:58+05:30 | End-to-end workflow contract evaluation report | `d6e61bdc477fbff3ee2c5f737b4759a30fb81503` |
| **SETUP-01** | Multi-Agent Collaboration Infrastructure Setup | Antigravity | `COMPLETED` | High | `main` | `docs/agent-handoff/`, root handoff files | None | File ownership & journal contract | Project Owner | 2026-11-27T11:02 / 2026-11-27T12:10 | Handoff directory, copied docs, verified hashes, 4 coordination files | `943ee72cce9dcaa0a0a06e76f75a7b4c229248f2` |
| **APO11-PROP** | Independent Research Quality Evaluation (Proposal) | Claude (Proto) / Codex (Core) / Antigravity (UI) | `PROPOSED` | High | TBD (separate worktrees upon approval) | TBD | SETUP-01 completion & Owner approval | Evaluation Protocol v1 (unapproved) | Project Owner + Claude | 2026-11-27T11:15 (Proposed) | Protocol design, Rust evaluator, persistence, Research OS inspection UI | None *(Not Started)* |

---

## 4. File Ownership Registry

To prevent overlapping edits and merge collisions, files and directories are assigned strict ownership during active development.

| Path / Directory | Assigned Owner | Access Permission | Active Task | Allowed Reviewers | Scope |
|---|---|---|---|---|---|
| `docs/agent-handoff/AGENT_SHARED.md` | Antigravity | Exclusive Routine Writer | SETUP-01 | Project Owner, Claude, Codex | Central shared coordination ledger |
| `docs/agent-handoff/agents/AGENT_CLAUDE.md` | Claude / Antigravity (as proxy) | Exclusive Writer | Idle | Project Owner | Claude's dedicated journal |
| `docs/agent-handoff/agents/AGENT_CODEX.md` | Codex | Exclusive Writer | Idle | Project Owner, Antigravity | Codex's dedicated journal |
| `docs/agent-handoff/agents/AGENT_ANTIGRAVITY.md` | Antigravity | Exclusive Writer | SETUP-01 | Project Owner | Antigravity's dedicated journal |
| `docs/agent-handoff/AETHER_*` | Read-Only | Read-Only | None | All Agents | Frozen handoff copies |
| `crates/aether-core/` | Codex | Exclusive Implementation | Idle | Claude, Antigravity, Owner | Rust core domain types & traits |
| `crates/aether-store/` | Codex | Exclusive Implementation | Idle | Owner | SQLite persistence layer |
| `crates/aether-engine/` | Codex | Exclusive Implementation | Idle | Claude, Owner | Evaluator & orchestration engine |
| `crates/aether-runner/` | Codex | Exclusive Implementation | Idle | Owner | Execution runner & native ops |
| `crates/aether-api/` | Codex | Exclusive Implementation | Idle | Antigravity, Owner | Backend HTTP/IPC API contracts |
| `crates/aether-cli/` | Codex | Exclusive Implementation | Idle | Owner | CLI binary commands |
| `apps/research-os/` (frontend) | Antigravity | Exclusive Implementation | Idle | Owner, Claude | React/TypeScript Research OS UI |
| `contracts/` (JSON schemas) | Shared (Explicit Agreement Required) | Shared Contract | Idle | All Agents, Owner | Canonical API & event schemas |
| `research/` & `docs/archive/` | Frozen / Project Owner | Read-Only (Owner Approval for Edits) | Idle | All Agents | Immutable research & archive files |

---

## 5. Global Activity Summary

All meaningful cross-agent events are recorded chronologically. Detailed progress is maintained in individual agent journals.

- **[2026-11-27T11:02:00+05:30] LOG-INIT-001 (Antigravity):** Received TASK 01 directive for multi-agent coordination setup.
- **[2026-11-27T11:10:00+05:30] LOG-DISC-002 (Antigravity):** Scanned repository and host environment. Located `AETHER_FULL_CONTEXT_AND_PROGRESS_REPORT.md` and `AETHER_MULTI_AGENT_QUICKSTART.md`. Confirmed `AETHER_AGENT_HANDOFF.zip` is missing from disk. Verified Git HEAD at `d6e61bdc477fbff3ee2c5f737b4759a30fb81503` on branch `main`.
- **[2026-11-27T11:15:00+05:30] LOG-COPY-003 (Antigravity):** Created `docs/agent-handoff/` and `docs/agent-handoff/agents/`. Copied `AETHER_FULL_CONTEXT_AND_PROGRESS_REPORT.md` and `AETHER_MULTI_AGENT_QUICKSTART.md`. Verified identical SHA-256 hashes before and after copying. Original files untouched.
- **[2026-11-27T11:15:30+05:30] LOG-LEDG-004 (Antigravity):** Created canonical `AGENT_SHARED.md` coordination ledger and initialized the 3 individual agent work journals (`AGENT_CLAUDE.md`, `AGENT_CODEX.md`, `AGENT_ANTIGRAVITY.md`).
- **[2026-11-27T11:42:00+05:30] LOG-CORR-005 (Antigravity):** TASK 01.1 correction: updated historical milestone descriptions in Global Task Board to authoritative text and established implementation sequence as APO-07 → APO-09 → APO-08 → APO-10. Preserved all prior log entries. Historical records remain reported history and not independently verified.
- **[2026-11-27T11:49:00+05:30] LOG-AUDIT-006 (Antigravity):** TASK 01.2 integrity audit: independently inspected Git commit history for APO-07, APO-09, APO-08, and APO-10. Verified commit existence, commit messages, and changed file lists. Corrected owned paths from hypothetical crates to verified modified paths (primarily `crates/aether-api/`, `web/`, `tests/`, `docs/decisions/`). Marked unverified historical primary owner, external reviewer, and start times as `UNKNOWN — NOT YET VERIFIED` per policy. Preserved handoff file hashes.
- **[2026-11-27T11:58:00+05:30] LOG-WT-007 (Antigravity):** TASK 01.3 worktree and branch readiness audit: executed `git worktree list --porcelain` and `git branch --list`. Confirmed only `main` branch exists. Verified proposed worktrees `..\AETHER-apo11-codex` and `..\AETHER-apo11-antigravity` do not exist on disk (PLANNED / NOT CREATED). Reconciled planned branch naming to `work/apo11-codex` and `work/apo11-antigravity`. Documented that separate worktrees do not automatically synchronize Markdown files.
- **[2026-11-27T12:10:54+05:30] LOG-CMT-008 (Antigravity):** TASK 01.4 checkpoint commit created on `main`: `943ee72cce9dcaa0a0a06e76f75a7b4c229248f2` ("docs: add AETHER multi-agent handoff and coordination system"). Staged exactly 8 permitted documentation files; confirmed 0 source files modified; verified clean working tree.

---

## 6. Shared API and Interface Contracts

Cross-agent implementations depend on mutually agreed, immutable contracts. No unapproved interface change may be treated as final.

| Contract ID & Version | Contract Name | Contract Owner | Consumer Agents | Schema / Interface References | Compatibility Rules | Approval Status |
|---|---|---|---|---|---|---|
| `CTR-EVT-01` (v1.0) | Claim & Event Contract | Codex | Codex, Antigravity, Claude | `contracts/claim.schema.json`, `contracts/event.schema.json` | Backward-compatible JSON schema | `APPROVED` (APO-09 baseline) |
| `CTR-WRK-01` (v1.0) | Worker Job & Result Contract | Codex | Codex, Runner | `contracts/worker-job.schema.json`, `contracts/worker-result.schema.json` | Content-addressed input/output digest binding | `APPROVED` (APO-10 baseline) |
| `CTR-EVAL-02` (Draft) | APO-11 Research Quality Evaluator API | Codex (Impl) / Claude (Design) | Codex, Antigravity | Proposed endpoints: `POST /api/v1/eval/quality`, `GET /api/v1/eval/reports/:id` | Semantic versioning; explicit null/abstain handling | `PROPOSED` (Unapproved; awaiting Owner sign-off) |

---

## 7. Architecture Decision Register (ADR)

*Rule:* Suggestions and proposals must never be represented as approved decisions.

| Decision ID | Proposal Description | Proposer | Reviewers | Project Owner Approval | Date | Affected Components | Relevant ADR Link |
|---|---|---|---|---|---|---|---|
| `ADR-ARCH-001` | Multi-Agent Coordination Infrastructure & Worktree Model | Antigravity / Owner | Claude, Codex | **APPROVED** | 2026-11-27 | `docs/agent-handoff/`, Git workflows | [ADR Traceability](file:///C:/Users/2k25c/PROJECTS/AETHER/docs/archive/v0.2/AETHER/19_ADR_TRACEABILITY.md) |
| `ADR-PROP-002` | APO-11 Independent Research Quality Evaluation Architecture | Claude / Antigravity | Codex, Owner | **PROPOSED** (Not Approved) | 2026-11-27 | Evaluator engine, SQLite schemas, Research OS UI | Proposed future milestone |

---

## 8. Blockers and Dependencies

| Blocker / Dependency ID | Description | Impacted Scope / Tasks | Assigned Owner | Resolution Status | Next Action |
|---|---|---|---|---|---|
| **BLK-DOC-001** | `AETHER_AGENT_HANDOFF.zip` missing from filesystem | SETUP-01 Handoff Bundle | Antigravity / Owner | **OPEN / DOCUMENTED** | Reported in final Task 01 report; proceed with existing unzipped handoff files |
| **BLK-MET-002** | Scientific-quality metrics unavailable; evidence unverified | Future scientific claims | Claude / Codex | **OPEN / BY DESIGN** | Requires APO-11 protocol and implementation before making claims |
| **BLK-TST-003** | Nine containment tests ignored by default in test suite | Full containment verification | Codex | **OPEN / KNOWN** | Address in dedicated containment milestone |
| **BLK-SQL-004** | Narrow SQLite journal-mode adjustment pending review | Release smoke consistency | Codex | **OPEN / MINOR** | Review SQLite pragmas during core maintenance pass |
| **BLK-TMP-005** | Temporary fixture database directory remains under `%TEMP%` | Test hygiene | Codex | **OPEN / MINOR** | Add deterministic fixture cleanup in future test harness pass |

---

## 9. Coordination Rules

All participating agents (Claude, Codex, Antigravity) and human contributors must strictly obey these thirteen rules:

1. **Read Ledger First:** Before starting any work, an agent must read the latest `AGENT_SHARED.md` from the canonical path.
2. **Read Individual Journal:** The agent must inspect its own individual journal file before taking action.
3. **Claim Before Modifying:** An agent must formally claim a task in the task board before modifying any project files.
4. **Declare File Ownership:** Agents must explicitly declare the files and directories they intend to modify.
5. **Review Before Duplicating:** Agents must review existing implementation, schemas, and prior art before creating new code.
6. **Respect Exclusive Paths:** Never modify a file or directory assigned exclusively to another agent.
7. **Freeze Shared Contracts:** Agree on shared API contracts and schemas before cross-agent implementation commences. No unapproved contract change is final.
8. **Record Meaningful Progress:** Log all major milestones, file modifications, and handoffs in the activity logs.
9. **Report Blockers Immediately:** Document blockers, missing dependencies, or test failures as soon as discovered.
10. **Never Overwrite Work:** No agent may delete, overwrite, or revert another agent's work without explicit Owner approval.
11. **Isolated Git Worktrees:** All code implementation must occur in isolated Git worktrees (proposed: `..\AETHER-apo11-codex` on branch `work/apo11-codex`, and `..\AETHER-apo11-antigravity` on branch `work/apo11-antigravity`; currently **PLANNED / NOT CREATED**). Never share one mutable working tree.
12. **Authorization Required:** Never commit to main, push to remotes, deploy, alter frozen research artifacts, or change project architecture without explicit Project Owner approval.
13. **Scientific Integrity:** Never promote exploratory claims to verified scientific facts. Preserve null results, contradictory evidence, and explicit uncertainty.

---

## 10. Write Ownership and Synchronization Protocol

### 10.1 `AGENT_SHARED.md` Maintenance
- **Designated Coordinator:** Antigravity is the designated routine writer and maintainer.
- **Authority:** The Project Owner retains absolute authority to edit, approve, or override this document.
- **Access Rule:** Claude and Codex must not directly write to `AGENT_SHARED.md`. They provide structured status updates in their own journals or via messages to Antigravity / Project Owner.
- **Reconciliation:** Antigravity reconciles agent updates into the global task board, activity summary, and blockers list.

### 10.2 Individual Agent Journals
- `docs/agent-handoff/agents/AGENT_CLAUDE.md`: Owned by Claude. When Claude is operating via web chat without local filesystem access, Antigravity or the Project Owner acts as scribe to paste Claude's structured updates.
- `docs/agent-handoff/agents/AGENT_CODEX.md`: Owned exclusively by Codex for its backend tasks.
- `docs/agent-handoff/agents/AGENT_ANTIGRAVITY.md`: Owned exclusively by Antigravity for coordination and UI tasks.
- *Strict Boundary:* No agent may edit another agent's journal.

### 10.3 Canonical Path Discipline & Non-Synchronization Notice
- **Canonical Path:** `C:\Users\2k25c\PROJECTS\AETHER\docs\agent-handoff\`
- **Original Integration Workspace:** The original `AETHER` checkout (`C:\Users\2k25c\PROJECTS\AETHER`) remains the integration workspace, the base for merges, and the canonical home of all coordination documents.
- **No Automatic Markdown Synchronization:** Separate Git worktrees do NOT automatically synchronize Markdown documents across checkouts. Agents operating in separate worktrees must read from and write to the canonical repository path (`C:\Users\2k25c\PROJECTS\AETHER\docs\agent-handoff\`) or communicate updates directly to the designated coordinator (Antigravity).
- **Web Chat Agent Access:** Claude operates via regular web chat and has no direct filesystem access; context is manually transferred by the Project Owner.
- **Integration Authority:** The Project Owner retains exclusive control over final integration and merging into `main`.

---

## 11. Handoff Protocol

Whenever an agent completes a task or milestone, it must submit a structured handoff report containing:
1. **Task ID & Objective:** What was assigned and delivered.
2. **Files Changed:** Exact repository-relative paths created or modified.
3. **Commit Hash:** Exact Git commit hash of the worktree branch.
4. **Validation Executed:** Exact tests, linters, and verification checks performed with pass/fail counts.
5. **Known Limitations:** Any edge cases, unverified assumptions, or stubbed logic.
6. **Remaining Blockers:** Unresolved issues requiring attention.
7. **Next Agent Action:** Specific agent nominated for the next phase.
8. **Required Interfaces:** Contracts, data types, or endpoints the next agent depends on.

The receiving agent must verify this report before beginning dependent work.
