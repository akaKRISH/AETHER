# 03 | Product requirements document (PRD)

## Product experience
A user enters a bounded objective and selects a task pack. AETHER presents a protocol, baseline, resource estimate, approved tools and pass conditions. The user approves. The system runs candidates, displays metrics and evidence, schedules verification, and exports a report with supported, rejected and uncertain claims. Cancel and inspect are always available.

| ID / release | Requirement | Acceptance criterion |
|---|---|---|
| PRD-F01 / R0 | Create project and task | Invalid task or budget rejected; valid config is persisted and hashed. |
| PRD-F02 / R0 | Freeze protocol and approve run | No worker starts before matching approval; edits require new version. |
| PRD-F03 / R0 | Execute baseline | Run produces manifest, logs, metric JSON and exit status. |
| PRD-F04 / R0 | Review evidence | Each exported quantitative claim links to immutable artifact and metric. |
| PRD-F05 / R0 | Stop and recover | Cancel stops active worker; restart resumes only eligible jobs. |
| PRD-F06 / R1 | Generate candidate hypotheses | Schema-valid proposals with rationale, expected metric and estimated cost. |
| PRD-F07 / R1 | Allocate research budget | Reservations never exceed approved limits; expired budget stops scheduling. |
| PRD-F08 / R1 | Independent verification | Fresh process and frozen evaluator; a failed replay prevents verification. |
| PRD-F09 / R1 | Export evidence bundle | Export includes protocol, code hashes, all runs, claims, verdicts and caveats. |
| PRD-F10 / R2 | Interactive web workspace | Accessible run list, budget, evidence links, compare runs, cancel button. |
| PRD-F11 / R2 | Remote GPU worker | Explicit opt-in; task and artifact checksums validated, no secrets in job. |
| PRD-F12 / R3 | Contributor SDK | New approved task adapter runs contract tests without changes to core. |

## Priorities and non-goals
Must: F01-F05 and minimum security tests. Should for research release: F06-F09. Could: F10-F12 only after evaluation evidence. No social network, universal chatbot, paid billing, 3D graphics, automated paper submission or fully automatic external access. No feature earns priority solely because it sounds advanced.

## Product measures
Time to first verified baseline (target <= 30 minutes after setup on small task); report provenance coverage target 100% for VERIFIED claims; reproducibility pass rate reported with denominators; cancellation responsiveness; task completion and cost. Targets are engineering goals, not measurements. No automatic success claim based on a model judging its own report.

## Rust-first release constraint
R0 ships a native Rust CLI and local research engine; installing an optional Python worker is required only for task packs that use Python ML. The deterministic Rust fixture must run without Python.
