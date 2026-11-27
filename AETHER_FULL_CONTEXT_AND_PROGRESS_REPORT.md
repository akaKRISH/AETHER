# AETHER — Full Project Context & Engineering Progress Report

**Document type:** Agent handoff / project briefing  
**Prepared:** 27 November 2026  
**Intended readers:** Claude, OpenAI Codex, Google Antigravity, and human project owner  
**Repository:** `C:\Users\2k25c\PROJECTS\AETHER`  
**Latest reported local commit:** `d6e61bdc477fbff3ee2c5f737b4759a30fb81503` (APO-10)  
**Latest reported worktree state:** Clean  
**Evidence status:** This report consolidates project context and milestone reports supplied by the owner in conversation. It is not an independent inspection of the current repository. Verify the actual checkout, files, commit ancestry, and test results before making changes or repeating claims.

---

## 1. Executive summary

AETHER is an ambitious, Rust-first **Personal Intelligence OS** whose most distinctive capability is intended to be a **Scientific Discovery Partner**: a hybrid-intelligence research laboratory that helps formulate questions, discover and organize literature, construct evidence and claim graphs, identify hypotheses and counterevidence, design experiments, execute approved workflows, evaluate outcomes, and preserve a reproducible record of what happened.

The project is deliberately being built as a trustworthy research system rather than a chatbot wrapper. Its central engineering principles are:

- Reproducibility and deterministic behavior wherever feasible.
- Evidence provenance, explicit uncertainty, and traceable claim dependencies.
- Separation of exploratory work from confirmatory work.
- Explicit approvals and default-deny permissions.
- Bounded execution through reviewed native operations.
- Immutable/content-addressed definitions and receipts.
- Independent evaluation rather than self-certification.
- Honest abstention when evidence, compute, or verification is insufficient.

The project has progressed from core infrastructure to a multi-stage research workflow system. The latest reported milestone, **APO-10**, evaluated the workflow contract using controlled fixtures: 9/9 workflow fixtures and 7/7 adversarial checks passed; worker-report and receipt digests matched across two fresh processes. Scientific-quality metrics were intentionally reported as unavailable. No held-out research study or model training was run.

**Current strategic position:** The next recommended phase is not more general orchestration. It is an independently designed research-quality evaluation system (working proposal: APO-11), alongside small maintenance follow-ups for the APO-10 SQLite journal-mode adjustment and the temporary fixture database directory. APO-11 is a proposal, not an implemented or approved milestone.

## 2. Vision, product direction, and long-term destination

### 2.1 Product direction

The owner selected **C — Personal Intelligence OS** as the overarching direction, with a balanced design emphasis across the main product dimensions. The most distinctive capability is the **Scientific Discovery Partner**, designed as a **Hybrid Intelligence Laboratory**.

The long-term goal combines several ambitions over time:

1. A persistent personal intelligence environment.
2. A capable scientific discovery and research partner.
3. A research and experimentation laboratory.
4. A broader, extensible intelligence platform that can eventually integrate models, tools, data, and workflows under strong governance.

The owner prefers maximum originality, even if substantial development takes a year. The desired learning style is to build first and then learn the system deeply by reverse-engineering its components. The owner is learning the technical foundations along the way and wants explanations from first principles when learning is requested.

### 2.2 Intended research lifecycle

The intended end-to-end lifecycle is approximately:

1. Accept a research question and constraints.
2. Discover relevant literature and source material.
3. Ingest permitted materials while preserving exact source bytes/passages and rights information.
4. Extract entities, claims, relations, citations, and evidence passages.
5. Build a versioned evidence/claim graph, including dependencies, conflicts, duplicates, and counterevidence.
6. Generate candidate hypotheses with alternatives, falsification criteria, metrics, and resource assumptions.
7. Design experiments with explicit controls, budgets, provenance, and analysis plans.
8. Qualify datasets and execution recipes.
9. Execute only approved, bounded operations.
10. Evaluate outputs with independent checks and, where justified, held-out data under separate custody.
11. Preserve reports, receipts, failures, null findings, uncertainty, and reproducibility details.
12. Use results to inform future work without silently rewriting historical conclusions.

The system must never conflate a plausible generated answer, a successfully executed workflow, or a passing software test with a scientifically true conclusion.

## 3. Repository and development environment

### 3.1 Known local environment

- Windows development environment.
- Repository path: `C:\Users\2k25c\PROJECTS\AETHER`.
- Rust 2024 edition core and native CLI.
- SQLite-backed persistence.
- Authenticated loopback API.
- React + TypeScript frontend / Research OS console.
- Python is reserved for explicitly approved, isolated scientific worker use; it is not a general-purpose skill execution escape hatch.
- Known toolchain at an earlier checkpoint: Rust/Cargo 1.96.1, Git 2.54.0, Windows SDK 10.0.26100.9457, Visual Studio Build Tools 2026 18.9.2.
- Hardware previously reported: Intel Core i5-13420H laptop CPU and NVIDIA RTX 3050 Laptop GPU with 6 GB VRAM. Do not assume total system RAM or currently free RAM from these facts.

### 3.2 Startup (local development)

Previously documented PowerShell startup procedure:

```powershell
Set-Location 'C:\Users\2k25c\PROJECTS\AETHER'
$env:AETHER_TOKEN = [Convert]::ToHexString([Security.Cryptography.RandomNumberGenerator]::GetBytes(32))
$env:AETHER_TOKEN
.\scripts\start-local.ps1
```

Then open `http://127.0.0.1:8080` and use the token locally. **Never paste the token into chat, commits, screenshots, logs, or agent prompts.** This procedure is historical context; confirm current script names and startup requirements in the checked-out repository before use.

### 3.3 Project documentation to consult first

The owner has maintained a documentation-first project context. Start with the smallest relevant set:

- `AGENTS.md` — repository-specific coding-agent instructions.
- `PROJECT_CONTEXT.md` — project context and architectural constraints (if still current).
- `PROJECT_STATUS.md` — current implementation status.
- `RESUME.md` — concise handoff and latest progress.
- `EXECUTION_QUEUE.md` / `NEXT_ACTIONS.md` — prioritized next work (use whichever is current).
- `docs/AETHER_MASTER_BLUEPRINT.md` — broader system blueprint.
- Relevant ADRs, especially `docs/decisions/ADR-021.md` and `ADR-022.md` for the skill registry and execution kernel, plus any later ADRs.
- Latest evaluation report: `research/results/apo-10-workflow-contract-evaluation.json`.

Do not repeatedly load the entire documentation inventory. Read the targeted files and relevant source modules needed for the assigned task. Check actual filenames and current status because the repository may have advanced since this handoff was assembled.

## 4. Architectural principles and system boundaries

### 4.1 Core architecture (as reported across milestones)

The architecture has evolved to include:

- Deterministic evaluator and protocol implementations.
- Transactional evidence store.
- ECAR (Evidence-Constrained Adaptive Research) scheduler.
- Supervised worker execution and authenticated loopback API.
- Browser console / Research OS frontend.
- Evidence graph export and inspection.
- Prometheus objective-bound evaluation protocol.
- Athena local model/research campaign infrastructure.
- Daedalus discovery and adaptive experiment-search work.
- Apollo literature, archive, knowledge graph, hypothesis, experiment-design, model-lab, and dataset-qualification subsystems.
- Declarative skill registry, bounded execution kernel, and deterministic workflow orchestration.

### 4.2 Trust and provenance model

The project distinguishes at least the following states and concepts:

- A source or passage can be stored without its claims being verified.
- A structurally valid evidence reference is not necessarily source-verified.
- A generated hypothesis is a proposal, not a finding.
- An experiment design is not an executed experiment.
- A local approval receipt is a record of operator consent, not an externally signed cryptographic proof.
- A deterministic execution receipt proves what the software recorded about execution; it does not prove the scientific truth of the result.
- A reproducible benchmark fixture establishes behavior on that fixture, not general scientific competence.
- Exploratory results must not be promoted to confirmatory evidence without a valid, independently controlled protocol.

### 4.3 Permission and execution model

- Default-deny capabilities.
- Explicit operator approval for the exact immutable definition/version being approved.
- Exact skill/workflow digest binding.
- Typed input/output schemas and mappings.
- Reviewed Rust-native operation allowlist.
- Bounded payloads, operation counts, dependency depth, and budgets.
- Content-addressed execution and workflow receipts.
- Deterministic sequential workflow order in APO-08.
- No general-purpose arbitrary-code execution through skills.
- No unrestricted network/filesystem privileges.
- No downloaded code or dynamically loaded plugin binaries.

**Important limitation:** Do not claim that this constitutes a complete OS sandbox. The current execution design has cooperative cancellation and does not provide hard timeout preemption or OS-level isolation. Historical Windows Job Object containment limits exist for reviewed worker paths, but the workers retained desktop identity and did not have OS-level filesystem/network denial or a restricted identity. Distinguish those containment controls from APO-09's skill execution guarantees.

## 5. Current reported repository state

| Item | Latest reported value |
|---|---|
| Repository | `C:\Users\2k25c\PROJECTS\AETHER` |
| Latest milestone | APO-10 — End-to-End Research Workflow Evaluation |
| Latest reported commit | `d6e61bdc477fbff3ee2c5f737b4759a30fb81503` |
| Worktree | Clean at report time |
| APO-08 commit | `d20d0a755ded1cbcd2f19389620bc8b995399b54` |
| APO-09 commit | `ef4f5bf` (kernel implementation commit `6c507e5`) |
| APO-07 commit | `b400904eecc18c4dadbeae37daeabcc31a57b338` |
| APO-10 report | `research/results/apo-10-workflow-contract-evaluation.json` |
| Frozen research/archive identities | 136 existing files reportedly retained their Git blob identities in APO-10 |

The current branch name, remote status, and exact commit ancestry were not provided in the latest report. Verify them using `git status`, `git branch --show-current`, `git log`, and `git worktree list` before parallel work. Do not assume that the local repository is pushed or that any commit exists on a remote.

## 6. Milestone history and engineering report

This section preserves the significant reported milestones and findings. Counts and values are historical snapshots, not necessarily current aggregate counts. Each result should be traced to its commit, report, fixture, and source code before reuse in a formal publication or investor/recruiter-facing claim.

### 6.1 Genesis and core system

**Genesis — `c017c05`**

Reported implementation:
- Deterministic evaluator.
- Transactional evidence store.
- ECAR scheduler.
- Supervised workers.
- Authenticated loopback API.
- Browser console.
- Evidence graph PNG export.
- Initial local fixture: 3/4.
- Fresh-process reproducibility demonstrated for the initial fixture.
- Windows SDK build environment issue fixed using Windows 11 SDK 10.0.26100.9457.
- Reported validation: 33 Rust tests + 3 frontend tests.
- Repository inventory at the time listed 107 requested documents.
- Concise-code policy was added to four project documents.

### 6.2 Prometheus — deterministic objective evaluation

**Commit/tag:** `d3586bf` / `prometheus-windows-verified`

- Max-subarray evaluation: Kadane's algorithm accepted at 623 and 1,131 cases in the reported validation contexts.
- Objective-bound protocol v3.
- Protocol report byte-for-byte reproducible.
- Reported validation: 50 Rust + 3 frontend tests.

### 6.3 Athena — local model and research campaign work

**Commit/tag:** `608163d` / `athena-windows-local`

- Local Ollama, cloud-disabled, Qwen 3.5 4B Q4_K_M on the RTX 3050 Laptop GPU.
- Warm time-to-first-token median reported around 224–232 ms.
- Device-wide sampled VRAM reported at 3,775 MiB.
- ECAR pilot utility difference: +0.17 points, 95% interval −0.84 to +1.19. This interval does not establish superiority.
- Synthetic development evidence only; no held-out validation at that stage.
- Reported tests: 53 Rust + 3 frontend.

**ATH-01 campaign — `7bef4a9` / `athena-campaign-01`**

- Authorized digest and explore/replay/challenge paths matched their references.
- Replay input digest and distinct challenge were checked.
- Trusted verifier verified one claim and a follow-up was proposed.
- 3/5 attempts; 140/12,000 ms reported.
- Export SHA-256 prefix/suffix: `7b1d9d...e6ad2`.
- Export was byte-identical across the reported reproducibility check.

**ATH-02 independent review — `ae259aa`**

- Found that 180 seed/stream pairs collapsed into 62 draw keys.
- Runner rejected held-out data, but analyzer accepted missing cases.
- Independent private inputs were required.
- No held-out data was generated or inspected.

**ATH-02B — `79132d8`**

- Split-seed family claim stream draw keys.
- Manifest digest bound protocol, executable, configuration, and analysis.
- Analyzer rejected missing, duplicate, unexpected, and unmatched cases.
- Independent custodian protocol added.
- No held-out data generated or inspected.

**ATH-04 — `e0bcc3a`**

- Windows Job Object limits for reviewed workers: 1 second user CPU per process/job, 64 MiB aggregate committed memory, one active process.
- Fail-closed behavior on setup/attachment failures.
- Inherited environment cleared; alternate executables and unexpected task fields rejected; workers reaped on timeout/cancellation.
- Tests covered CPU quota, memory/child process denial, job cleanup, secret exclusion, timeout, and cancellation.
- Crucial residual limitation: workers still used desktop identity and did not have OS-level filesystem/network controls or restricted identity.

**ATH-05B — `09f17f3` / `athena-engineering-verified`**

- Playwright 1.65.1 and existing Chromium 1255 cache used for desktop/mobile browser smoke.
- Timestamp validation tests 3/3.
- Documentation validation rejected 5 negative schema cases.
- `npm ci`, lint, type-check, tests (3/3), build passed.
- Frozen artifact hashes unchanged.
- This was engineering verification, not held-out scientific validation.

### 6.4 Daedalus — discovery and adaptive search experiments

**DAE-01 — `7f9c1db`**

- Deterministic 12-item knapsack discovery.
- Typed candidates, one-bit mutation, diverse parent selection, lineage, exhaustive oracle, SQLite SHA-256 report.
- 42 tests; exploratory.

**DAE-02/03 — `05aac30`**

- 36 matched development comparisons.
- Greedy exact: 23; evolution exact: 1; random exact: 0.
- Mean gaps: 1.42, 60.83, 85.92 respectively.
- Small synthetic grid; timing used fixed order in one Windows run.
- 108 arm records, 1,728 candidate evaluations.
- Held-out evaluation deferred.

**DAE-04 — `2567242`**

- Greedy seeding improved evolution in 32/40 pairs.
- Local improvement helped 15 and hurt 1.
- Adaptive hybrid beat fixed hybrid in 3 comparisons and tied in 37.
- Development evidence only.

**DAE-05 — `88f147e`**

- Posterior reliability weighting tied fixed prior utility across 480 matched cases.
- Lost two cases to full downstream evaluation.
- Compared with original ECAR: +3.24 on unreliable hubs but −1.45 overall.
- Scheduler timing 3.83 µs vs 3.61 µs in one Windows run.
- Synthetic evidence.

**DAE-05B — `170d354`**

- Posterior changed rankings in 152/360 cases but improved utility in only 2.
- Richer evidence reduced Brier score but did not establish verified utility improvement.
- Information bonus helped Beta(1,1), hurt Beta(4,4), and tied Beta(12,12).
- Four copies/source group produced identical decisions in the reported analysis.
- Synthetic receipts/copies.

**DAE-06 — `14bc64d`**

- Adaptive test-time compute controller with budgets, lineage, repetition stopping, and cancellation.
- Deterministic fixtures: adaptive and best-of-three each verified one answer, using 8 vs 9 calls.
- Live Qwen pilot: all strategies abstained after verifier rejected the same suboptimal interval.
- 10 calls, 821 prompt tokens, 210 output tokens, 23 seconds; reasoning tokens unreported.
- Fixed strategy order and one live task; not general evidence of model superiority.

**DAE-07 — `60830db`**

- Release performance suite with 12 workloads.
- API p50/p95: 5.38/7.21 ms.
- Report p50/p95: 9.80/12.35 ms.
- Worker startup p50/p95: 51.29/58.34 ms.
- Parent working set: 21.2 MB.
- CPU measurement was coarse and excluded children; p99 withheld with fewer than 1,000 samples.

**Rune exploration / DAE-10 preparation**

- Evaluated `anomalyco/rune` in an isolated workspace.
- Build was aborted when free RAM approached 120 MiB; compiler working set was around 2.09 GB.
- 435 dependencies; workspace metadata passed.
- Only a hello/starter example was reached; no AETHER prototype resulted.
- Probe was kept separate and unqualified.

**DAE-10 — `a18c1b8` / `daedalus-engineering-verified`**

- 77 workspace tests passed; 9 ignored.
- Strict Clippy, formatting, release build, frontend 5/5, docs, smoke passed.
- Seven frozen research hashes unchanged.
- Authentication and containment tests passed.
- No held-out scientific evidence; Rune remained unqualified.

### 6.5 Apollo — literature, evidence, hypotheses, experiments, and model lab

**APO-01 — `3bac54c`**

- Literature discovery caches/resumes Crossref, OpenAlex, and arXiv.
- Revisions and citations persisted in SQLite.
- CLI/API/Research OS searchable archive.
- UTF-8 local intake with rights confirmation and exact byte/passage preservation.
- Claims remained unverified; providers used fixtures and a synthetic document; PDF ingestion deferred.
- 30 Rust + 5 frontend tests.

**APO-02 — `bed6758`**

- Persistent SQLite graph of typed entities, passages, versioned relations, citations, dependencies, conflicts, and duplicates.
- Only trusted campaign snapshots provided verified receipts.
- 35 Rust + 5 frontend tests.
- A 65-node fixture returned 64/63 and marked truncation.
- Fresh exports had identical SHA values.
- Two licensed excerpts were checked; PDF and live provider qualification pending.

**APO-03 — `b89d3b8`**

- Hypothesis discovery from bounded graph, including source evidence/counterevidence, alternatives, falsification, metrics, and resource estimates.
- 20 archived DAE-04 comparisons at 8 evaluations/arm: adaptive hybrid won 2, tied 13, lost 5 versus greedy.
- Three exploratory proposals; no comparable contradiction.
- Reports byte-identical.

**APO-04 — `77e135a`**

- Automated experiment design generated three unapproved exploratory designs with matched budgets, source provenance, falsification criteria, and canonical SHA-256.
- Two CLI processes produced byte-identical reports.
- 46 Rust API/platform + 5 frontend tests; gates passed.
- Execution blockers: reviewed paired-knapsack worker/verifier, fresh cases, and a missing local-plus-repair factorial arm.
- Confirmatory work required independent custody and justified sample assumptions.
- No experiment or held-out study ran.

**APO-05 — `ff48125`**

- Rust model-lab CLI/API/Research OS.
- Four installed GGUF headers inspected; full Qwen 3.5 4B SHA-256 verified; SafeTensors fixtures used.
- Scripted baseline 7/7; degraded fixture 5/7, exposing calibration/refusal regressions.
- Live pilot was prevented before dispatch due to 2.71 GiB available RAM versus a 5.10 GiB admission estimate; 5,685 MiB free VRAM was reported.
- No live model token/latency evaluation ran in that pilot.
- 66 Rust + 5 frontend tests.
- Scripted scores are not actual model-intelligence scores.

**APO-06A — `0fb3662`**

- Training controller, pinned LoRA/QLoRA proposals, split-contamination checks, resource estimates, capability/safety regression gates.
- Rust/Python fixtures matched 4 completion/cancel receipts; zero weight updates.
- Actual training blocked: installed Qwen GGUF was not trainable by the selected backend; Python environment lacked PyTorch, Transformers, PEFT, and bitsandbytes.
- Hypothetical QLoRA estimate: 4.47 GiB RAM versus 1.47 GiB available.
- 57 Rust + 5 frontend tests.

**APO-06B — `5f30f71`**

- Deterministic development dataset qualification/export with provenance and group-aware splits.
- Two fresh processes produced manifest SHA-256 `0c52ab45…c8c1eb9`.
- 26/28 recipes retained; 18 train and 8 validation examples across 8 groups; all 5 requested topics represented.
- Two source-linked CC BY 4.0 excerpts.
- No independent test export or training authorized.
- Reported validation: 39 unit + 20 platform tests, strict API Clippy, formatting, API release, frontend 5 tests/lint/build, desktop/mobile smoke, docs.
- APO-06C–F were resource-blocked. APO-07/08/09 were prioritized before live training.

### 6.6 Skill registry, execution kernel, and workflows

**APO-07 — `b400904eecc18c4dadbeae37daeabcc31a57b338`**

Reported implementation:
- Five immutable, content-addressed declarative skills.
- Typed schemas.
- Default-deny capabilities.
- Pinned dependency composition.
- Deterministic preflight records.
- Local CLI review/approval.
- Authenticated API discovery.
- Research OS inspection view.
- Tests for schema errors, cycles, budgets, tampering, permission escalation, malicious instructions, evidence composition, and replay.
- Validation: 47 Rust unit tests, 20 platform tests, strict Clippy, formatting, optimized release build, frontend checks/tests, desktop/mobile smoke, documentation validation.
- Skills remained inert: no skill execution and no model call.
- Evidence references received shape checks only; no source verification.
- Local approvals were not externally signed.

**APO-09 — `ef4f5bf` (kernel implementation commit `6c507e5`)**

Reported implementation:
- Bounded execution kernel restricted to reviewed Rust operations.
- Pinned skill definitions, capability checks, bounded payloads, content-addressed receipts.
- Validation: 127 workspace tests; 9 containment tests intentionally ignored by default; strict Clippy, formatting, optimized release build, frontend lint/type-check/tests/build, desktop/mobile smoke, documentation validation.
- Evidence remained unverified; local approvals remained consent records, not cryptographic proof.
- Cancellation was cooperative; no hard timeout preemption or OS-level sandbox.
- Worktree clean at report time.

**APO-08 — `d20d0a755ded1cbcd2f19389620bc8b995399b54`**

Reported implementation:
- Content-addressed workflow DAGs.
- Exact workflow and skill approval binding.
- Typed mappings.
- Deterministic sequential execution through APO-09.
- Aggregate budgets.
- Replayable receipts.
- CLI/API support and read-only Research OS view.
- Validation: 136 workspace tests, strict Clippy, formatting, native release build, frontend lint/type-check/tests/build, desktop/mobile browser smoke, documentation validation.
- Nine existing Windows containment tests remained ignored by default; harness-driven containment checks ran.
- Evidence remained unverified; orchestration did not establish research correctness.
- Cooperative cancellation only; no hard preemption or OS-isolation claim.
- No held-out study or training.

**APO-10 — `d6e61bdc477fbff3ee2c5f737b4759a30fb81503`**

Reported implementation/evaluation:
- End-to-end research workflow contract evaluation.
- Content-addressed report at `research/results/apo-10-workflow-contract-evaluation.json`.
- 9/9 workflow fixtures passed.
- 7/7 adversarial checks passed.
- Worker-report and receipt digests matched across two fresh processes.
- Scientific-quality metrics explicitly unavailable; evidence remained unverified.
- Validation: 138 workspace tests; 9 existing containment tests ignored by default, with harness checks reported as passed; strict Clippy; formatting; optimized release build; frontend lint/type-check/tests/build; desktop/mobile browser smoke; documentation validation.
- All 136 existing research/archive files reportedly retained their Git blob identities.
- Release smoke first encountered a SQLite lock; a narrow journal-mode adjustment was made, then final release smoke passed.
- One temporary fixture database directory remained under `%TEMP%` because cleanup was blocked by tool policy.
- No held-out study, training, or scientific correctness claim.

## 7. Current APO-10 interpretation

### What the results support

- The tested workflow fixtures satisfied their expected contract.
- The tested adversarial cases were rejected or handled as designed.
- The tested worker report and receipt artifacts were reproducible across two fresh processes.
- The implementation passed the reported engineering validation gates.
- Existing frozen research/archive file identities were preserved according to the reported Git-blob comparison.

### What the results do not support

- They do not establish scientific validity or novelty.
- They do not show that literature coverage is complete.
- They do not prove that citations semantically support claims.
- They do not demonstrate that generated hypotheses are useful or falsifiable in real scientific domains.
- They do not establish general robustness beyond the nine workflow fixtures and seven adversarial checks.
- They do not establish security against a hostile local user or OS-level compromise.
- They do not establish safe hard cancellation or preemption.
- They do not establish model intelligence or fine-tuning effectiveness.

## 8. Known constraints, caveats, and unresolved risks

### 8.1 Scientific-method risks

- No independent held-out research study has been completed in the reported history.
- Some experiments are synthetic or based on limited development grids.
- Historical tests sometimes show small samples or fixed arm order; do not infer broad superiority.
- ECAR superiority has not been established. A prior ECAR pilot interval included zero (−0.84 to +1.19 around +0.17), and later synthetic comparisons had mixed outcomes.
- DAE-05/05B show that posterior/ranking changes do not automatically improve utility.
- DAE-06 live model pilot abstained after a verifier rejected the candidate answer; this is a useful safety result, not evidence of strong model reasoning.
- Benchmark quality must be separated from workflow execution correctness.
- A self-evaluator must not be treated as independent merely because it is a separate module. Independence needs a separate specification, data custody, oracle/human review, and controls against shared generator bias.

### 8.2 Security and execution risks

- APO-09 uses cooperative cancellation; a stuck native operation may not be forcibly stopped by the kernel.
- No OS-level sandbox or hard preemption is claimed.
- Historical Windows Job Object limits constrain CPU/memory/process count but do not amount to filesystem/network isolation or restricted identity.
- Local approvals are not externally signed.
- Evidence references remain unverified absent an authorized source-verification process.
- The execution allowlist and skill registry require careful review for permission escalation and version/digest binding.

### 8.3 Resource constraints

- The owner’s laptop has an i5-13420H and RTX 3050 Laptop GPU with 6 GB VRAM.
- Local model/training workloads have previously been blocked by RAM admission and missing Python training dependencies.
- APO-05's live model pilot was blocked at 2.71 GiB free RAM versus 5.10 GiB estimated need.
- APO-06A estimated 4.47 GiB RAM for a hypothetical QLoRA path while 1.47 GiB was available.
- Do not recommend installing large dependencies, building large workspaces, or starting training without checking current memory, disk, GPU, environment, and user approval.
- Cloud credits were mentioned as a possible resource, but no active cloud environment or authorization is established here.

### 8.4 APO-10 maintenance notes

1. Inspect the narrow SQLite journal-mode adjustment and add a targeted regression test for the lock scenario. Do not revert it reflexively; understand the concurrency and durability implications first.
2. Identify the exact temporary fixture database directory under `%TEMP%` before cleanup. Confirm it is the artifact from this run and not user data. Remove only that precise path after validating ownership and no active process handles it.
3. Confirm the 136-file Git-blob identity procedure is documented and reproducible.
4. Keep the nine intentionally ignored containment tests visible. Do not unignore them solely to inflate test counts; document the harness path used to run them.

## 9. Recommended next phase: APO-11 (proposal only)

**Working title:** Independent Research Quality Evaluation.  
**Status:** Proposed by the assistant; not yet approved, implemented, or committed.  
**Goal:** Evaluate the scientific quality and epistemic reliability of AETHER research outputs separately from the software workflow contract.

Potential dimensions:

1. **Literature coverage and source quality:** relevant-source recall, source-type diversity, publication/version provenance, rights status, and known omissions.
2. **Claim-evidence entailment:** whether each claim is directly supported, indirectly supported, contradicted, or unsupported by its cited passage.
3. **Contradictions and counterevidence:** whether credible alternatives and conflicting findings are represented rather than suppressed.
4. **Hypothesis quality:** specificity, novelty relative to a defined corpus, falsifiability, measurable predictions, and plausible alternative explanations.
5. **Experiment design quality:** controls, confounders, sample-size justification, pre-registration, outcome measures, stopping rules, and analysis plan.
6. **Uncertainty/calibration:** quality of uncertainty statements and abstention under missing/contradictory evidence.
7. **Reproducibility:** independently repeatable evaluation, exact inputs, versioned rubric, and auditable human adjudication.

### Required independence safeguards

- Separate the generator from the evaluator in code, data, and authority.
- Predefine rubrics, scoring rules, benchmark version, and success criteria before evaluating candidate outputs.
- Use independent human reviewers or trusted external oracles for ground-truth judgments where feasible.
- Blind reviewers to system identity/strategy where practical.
- Preserve disagreement and inter-rater reliability; do not hide ambiguous adjudications.
- Keep development examples separate from held-out examples.
- If held-out evaluation is eventually approved, use independent custody, immutable manifests, sealed inputs, access logs, and a pre-specified analysis plan.
- Do not let the model or generator select, inspect, or modify held-out cases.
- Avoid a single composite "intelligence score" unless its construction is justified and independently validated. Prefer dimension-specific metrics and confidence intervals.
- Do not claim scientific discovery based only on rubric scores.

### Suggested APO-11 stages

- **APO-11A — Evaluation protocol design:** define constructs, rubrics, reviewer workflow, benchmark taxonomy, data provenance, and statistical plan. No new model calls required.
- **APO-11B — Evaluator implementation:** implement versioned rubric engine, adjudication records, score provenance, uncertainty, and report export.
- **APO-11C — Development benchmark:** use synthetic or public development fixtures only, clearly labeled; test evaluator consistency and adversarial failure modes.
- **APO-11D — Independent review pilot:** only after protocol approval, conduct a small, pre-registered human review pilot; determine whether a held-out evaluation is justified.

This staged decomposition is a proposal. Inspect `EXECUTION_QUEUE.md` and current owner priorities before creating tickets or coding.

## 10. Multi-agent operating model: Claude + Codex + Antigravity

The owner intends to use all three tools simultaneously. This can increase throughput, but only if work is partitioned. **Never let two agents edit the same working tree at the same time.** Separate Git worktrees/branches or assign a single writer at a time.

### 10.1 Recommended roles

| Agent | Primary role | Best work types | Avoid |
|---|---|---|---|
| Claude | Architect, research-methodology critic, threat-model reviewer, independent design reviewer | Requirements, architecture alternatives, scientific evaluation protocols, adversarial test plans, code review, critique of Codex/Antigravity diffs | Uncoordinated edits to files another agent owns; inventing repository facts without reading files |
| Codex | Primary Rust/backend implementation and test owner | Rust core, SQLite/schema migrations, CLI/API, execution kernel, deterministic evaluation, integration tests, release gates | Simultaneously editing files assigned to Antigravity; broad refactors unrelated to the ticket |
| Antigravity | Frontend/Research OS integration, developer experience, docs/visual inspection, independent bounded implementation slice | React/TypeScript views, CLI/UI integration, browser smoke, documentation, isolated frontend components | Editing Rust/backend files assigned to Codex; changing APIs without an agreed contract |

Role allocation can change by milestone. If the next task is primarily scientific-method design, Claude should lead the protocol proposal while Codex and Antigravity wait or take separate, non-overlapping support tasks. Do not force all three agents to code on every task.

### 10.2 A safe parallel workflow

1. **Human owner chooses one milestone and its acceptance criteria.** Read the current `RESUME.md`, `PROJECT_STATUS.md`, and `EXECUTION_QUEUE.md` first.
2. **Claude produces an architecture/protocol proposal and risk review** in a read-only or separate branch. It must cite actual file paths and line numbers when making repository-specific claims.
3. **Agree on interfaces before splitting implementation.** Define API contracts, schema changes, file ownership, test requirements, and migration strategy.
4. **Create isolated worktrees from the same base commit.** Codex and Antigravity must not share a mutable checkout.
5. **Assign non-overlapping scopes.** Example: Codex owns Rust/API/schema; Antigravity owns frontend consuming the agreed API; Claude reviews the plan and diffs.
6. **Each agent commits only its own branch.** Each reports commit hash, changed files, tests run, tests not run, and limitations.
7. **Integrate serially.** One designated integration owner cherry-picks or merges one branch at a time, resolves conflicts, runs full gates, and updates project docs.
8. **Run a final independent review.** Claude reviews the integrated diff and test report; it does not merely trust another agent's summary.
9. **Human owner approves the final local commit.** Do not push, publish, deploy, install heavy dependencies, or change system-level settings without explicit permission.

### 10.3 Example PowerShell worktree setup

Run only after confirming the current repository is clean and the base commit is correct. Use unique branch and directory names per milestone; these are examples, not commands to run blindly.

```powershell
Set-Location 'C:\Users\2k25c\PROJECTS\AETHER'
git status --short
git log -1 --oneline

# Create isolated Codex and Antigravity worktrees from the current HEAD.
git worktree add -b agent/apo11-codex ..\AETHER-apo11-codex HEAD
git worktree add -b agent/apo11-antigravity ..\AETHER-apo11-antigravity HEAD

# Inspect all worktrees.
git worktree list
```

Claude can review a clean snapshot or use a separate review branch/worktree. If Claude is only proposing a protocol, prefer read-only review rather than giving it a writable checkout.

After integration and when no process is using a worktree, remove only the relevant worktree with `git worktree remove <path>` and delete its branch only after confirming it has been merged or is no longer needed. Do not use `git clean -fdx`, `git reset --hard`, or broad directory deletion as routine cleanup.

### 10.4 File ownership example for an APO-11 implementation

This is illustrative; adjust after reading the actual repo layout.

- **Claude (design/review):** protocol specification, scoring rubric, benchmark independence design, threat model, adversarial review. Prefer output as a proposal document, not direct code edits.
- **Codex (backend):** Rust evaluator types, persistence, APIs, canonical reports, tests, reproducibility.
- **Antigravity (frontend):** Research OS evaluation inspection UI, reviewer workflow, report visualization, browser tests. Consume a frozen API contract; do not independently invent backend fields.
- **Human owner/integration lead:** resolves disputes, approves scope and permissions, merges/cherry-picks, runs final gates, updates canonical project status.

If a task's backend and frontend cannot be cleanly separated, use one implementation agent and have the others perform review or test design rather than parallel code edits.

## 11. Global agent instructions (paste into Claude, Codex, and Antigravity)

```text
You are contributing to AETHER, a Rust-first Personal Intelligence OS and
Scientific Discovery Partner. Treat the repository's AGENTS.md and current
project status documents as authoritative over this handoff if they are newer.

First establish the actual checkout, current branch, HEAD, worktree state, and
relevant project instructions. Do not assume this handoff is perfectly current.

Preserve scientific integrity: never invent results, never upgrade exploratory
results to confirmatory evidence, never treat generated claims or shape-valid
evidence references as verified, and never claim scientific discovery merely
because a workflow or test passed. Preserve null findings and uncertainty.

Preserve security boundaries: default-deny capabilities; exact immutable
skill/workflow approval binding; only reviewed Rust-native operations through
the APO-09 kernel; no arbitrary shell/Python/JavaScript execution through skills;
no downloaded code, dynamic plugin binaries, unrestricted network/filesystem
access, unapproved model calls, or training. Do not claim hard cancellation,
OS isolation, or cryptographic approval where none exists.

Do not alter frozen research/archive artifacts unless the task explicitly
requires it and the owner approves. Do not read, generate, or inspect held-out
data without a separately approved custody protocol. Do not start training.

Use concise, idiomatic Rust; concise code means clear and focused, not code-golf.
Prefer targeted tests and small diffs. Avoid broad refactors, repeated full-doc
loads, unnecessary dependencies, and subagents unless justified.

Never push, publish, deploy, install heavy dependencies, modify system-level
settings, or perform destructive cleanup without explicit owner approval.
Never expose AETHER tokens, credentials, local private paths beyond what is
needed, or sensitive data in logs, commits, screenshots, or prompts.

Work only in the assigned branch/worktree and the assigned file scope. Do not
modify another agent's files. Before changing shared interfaces, propose the
contract and get agreement. Report exact commit hash, files changed, tests run,
tests skipped/ignored, failures, limitations, and remaining risks. Update
RESUME.md, PROJECT_STATUS.md, and EXECUTION_QUEUE.md as appropriate.

Any coding prompt must identify an appropriate available model, reasoning
effort, and Standard speed. Do not assume a model, quota, or subscription is
available; state a practical fallback.
```

## 12. Ready-to-use agent kickoff prompts

### 12.1 Claude — architecture and independent scientific-method review

```text
You are the independent architecture and research-methodology reviewer for
AETHER. Read this handoff, then inspect the current repository's AGENTS.md,
RESUME.md, PROJECT_STATUS.md, EXECUTION_QUEUE.md, APO-10 evaluation report, and
relevant ADRs. Treat newer repository documentation as authoritative.

Do not implement code yet. Produce an APO-11 design review that:
1. Separates workflow/software correctness from scientific output quality.
2. Defines measurable research-quality constructs and dimension-specific metrics.
3. Designs an independent evaluator that does not trust the generator's own
   claims, evidence status, or self-assigned scores.
4. Specifies human adjudication, disagreement handling, benchmark versioning,
   development/held-out separation, custody, blinding, and analysis controls.
5. Identifies threats to validity, leakage, circular evaluation, evaluator bias,
   source-quality confounds, and incentives to game the rubric.
6. Recommends the smallest defensible first milestone and its acceptance gates.
7. Identifies exact existing AETHER modules to reuse and interfaces that need
   clarification, with file paths and line references.

Do not invent repository state, benchmark results, scientific claims, or test
results. Label assumptions and unknowns. Return a concise architecture decision
proposal, risk register, test plan, and non-overlapping task breakdown for
Codex and Antigravity. Do not edit shared working files.
```

### 12.2 Codex — primary implementation prompt template

```text
Model: choose the best available coding model appropriate to task complexity;
Reasoning: Medium for bounded implementation, High for security/methodology;
Speed: Standard. State a fallback if the preferred model is unavailable.

Implement only the assigned AETHER workstream in this isolated worktree.
First inspect AGENTS.md, RESUME.md, PROJECT_STATUS.md, EXECUTION_QUEUE.md, the
relevant ADRs, and only the source modules required. Verify branch, HEAD, and
clean status. Do not reload all project documentation.

Own only the Rust/backend files and tests listed in the task assignment. Do not
modify Antigravity-owned frontend files or shared contracts without agreement.
Reuse current architecture. Keep the diff narrow, Rust idiomatic, deterministic,
and testable. Preserve all scientific, evidence, capability, and sandbox limits
in this handoff. Do not run held-out studies or training.

Run targeted tests during work and all required final gates. Preserve frozen
research/archive Git blob identities. Update project status/resume/queue only if
those files are in your assigned scope or integration owner requests it.
Commit locally. Report commit, changed files, tests passed/failed/skipped,
resource usage/blockers, limitations, and any manual integration steps.
Do not push or deploy.
```

### 12.3 Antigravity — frontend and Research OS prompt template

```text
Implement only the assigned Antigravity frontend/Research OS workstream in an
isolated worktree. First inspect AGENTS.md, current project status, the API
contract, and the relevant UI components. Verify branch, HEAD, and clean status.

Own only the assigned React/TypeScript/UI files and frontend/browser tests.
Consume the agreed backend contract exactly; do not invent API fields or change
Rust schemas. Do not edit Codex-owned backend files. If an API change appears
necessary, document the required contract change and stop at that boundary.

Keep the UI read-only unless the approved task explicitly calls for an action.
Clearly distinguish verified, unverified, exploratory, failed, abstained, and
incomplete states. Never imply scientific correctness from a workflow status
or visual score. Preserve accessibility, responsive desktop/mobile behavior,
and existing design conventions.

Run lint, type-check, frontend tests, build, and relevant desktop/mobile browser
smoke. Commit locally and report commit, changed files, tests, known gaps, and
integration requirements. Do not push or deploy.
```

## 13. Prompt and model-efficiency preferences

The owner wants coding-agent prompts to include a recommended model and reasoning effort matched to task complexity, with **Standard speed** as the default. Do not assume the exact model or quota is available. Provide a practical alternative.

General guidance:
- Routine, narrow implementation or UI work: a capable standard coding model with Medium reasoning and Standard speed.
- Security-sensitive execution, trust boundaries, independent evaluation design, or complex architecture: a stronger available reasoning model with High reasoning and Standard speed.
- Use Claude primarily for critique, research-method design, independent review, and architecture where that creates useful separation from the implementation agent.
- Use targeted continuation prompts. Do not repeatedly ask agents to reload the entire documentation set.
- Avoid unnecessary subagents and expensive parallel reasoning when work can be split by clear ownership.

## 14. Immediate suggested action plan

1. Verify APO-10 commit and worktree state in the real local repo.
2. Preserve the APO-10 report and its exact digest; do not overwrite it.
3. Make a small maintenance issue for the SQLite journal-mode regression test and a separate safe cleanup issue for the exact temporary fixture directory. Keep these separate from research-quality evaluation.
4. Ask Claude for the APO-11 independent evaluation protocol and risk review (read-only).
5. Have the human owner approve the protocol and milestone acceptance criteria before implementation.
6. Create isolated Codex and Antigravity worktrees from the same clean base commit.
7. Assign Codex Rust evaluator/persistence/API work; assign Antigravity read-only Research OS inspection/reviewer UI only after API contracts are agreed.
8. Integrate serially, run full gates, preserve the 136 frozen research/archive identities (and update this protected set only through explicit approval), and request Claude's independent review of the integrated diff and evaluation methodology.
9. Do not begin held-out studies, model calls, or training until the protocol, custody, resource admission, and explicit approvals are complete.

## 15. Glossary

- **AETHER:** Personal Intelligence OS / hybrid scientific discovery partner.
- **ECAR:** Evidence-Constrained Adaptive Research scheduler; its superiority is not established.
- **Evidence graph:** Versioned structure linking sources/passages, claims, entities, dependencies, conflicts, and citations.
- **Receipt:** Content-addressed record of a workflow/operation execution; not proof of scientific truth.
- **Content-addressed:** Identity derived from content digest so changes create a different identity.
- **Held-out data:** Evaluation data kept inaccessible to development and tuning processes until the pre-specified evaluation is run.
- **Independent custody:** A separate custodian controls hidden evaluation material and records access.
- **Exploratory:** Development-stage work used to generate ideas or estimate behavior; not confirmatory proof.
- **Confirmatory:** Pre-specified evaluation designed to test a defined claim under controlled analysis and custody.
- **Abstention:** Explicitly declining to make a claim when evidence or verification is insufficient.
- **Default-deny:** No capability is available unless explicitly permitted by the applicable reviewed policy and approval.
- **Worktree:** Separate Git checkout/working directory associated with a branch, allowing agents to work without editing the same files in one checkout.

---

## 16. Handoff disclaimer

This document is a comprehensive synthesis of the owner-reported project history available at preparation time. It does not replace repository truth. Every agent must verify the live checkout and treat more recent code, ADRs, status documents, manifests, tests, and report artifacts as authoritative. Where this document records metrics or milestones, those are reported results unless and until reproduced from the repository.
