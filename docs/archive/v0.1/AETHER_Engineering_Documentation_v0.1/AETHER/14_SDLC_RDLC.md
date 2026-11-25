# 14 | SDLC and research development lifecycle (RDLC)

## Two synchronized lifecycles
RDLC here means Research and Development Life Cycle (interpretation of the requested acronym); SDLC means Software Development Life Cycle. Product features and experiments share requirements, versioning and release gates, but research conclusions require a separate protocol and evidence review.

| Stage | Software deliverable | Research deliverable / exit gate |
|---|---|---|
| 0 Discover | Charter, PRD, prior art | Literature map; faculty review; falsifiable question. |
| 1 Specify | TRD, architecture, ADRs, contracts | Frozen metric definitions and pilot plan. |
| 2 Prototype | R0 CLI, baseline, artifact store | Fresh baseline replay; known failure fixtures. |
| 3 Pilot | Scheduler, evidence graph, verifier | Dev tasks; calibrate budget and sample size. |
| 4 Validate | Stable tagged code, QA/security gates | Preregistered held-out study; full logs. |
| 5 Release | Versioned package, docs and demo | Technical report, data card and reproducibility bundle. |
| 6 Evolve | Issue triage, compatibility, migrations | Follow-up questions; new protocol versions. |

## Engineering workflow
One issue per small deliverable; acceptance criteria and linked requirement. Branch -> code -> local tests -> review diff -> CI -> merge. Solo work still uses a self-review checklist and periodic mentor review. Commit only code/data permitted by license. Keep experiments separate from application code and immutable after registration. Never modify held-out evaluator to make a result pass; corrections require new version and transparent rerun.

## Definition of done
Readable code, type checks, unit/integration tests, updated documentation, linked requirement, dependency/license review, no secrets, recovery/failure behavior, and a human explanation of core algorithm. Research change additionally includes hypothesis, baseline, fixed protocol, raw artifacts, negative results and measured uncertainty. Approval to enter next phase is based on evidence, not elapsed time or number of features.
