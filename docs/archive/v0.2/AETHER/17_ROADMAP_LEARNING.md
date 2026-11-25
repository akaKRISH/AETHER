# 17 | Roadmap, learning curriculum and deliverables

## Twelve-month sequence (indicative, not a deadline guarantee)
| Months | Build milestone | Learn from scratch / proof |
|---|---|---|
| 1 | R0: CLI baseline, metrics and manifest | Rust variables, functions, structs, enums, ownership, borrowing, Result, Cargo and Git; explain baseline and seed. |
| 2 | Ledger, evidence schema and replay | SQL, hashing, JSON, tests; trace one claim to artifacts. |
| 3 | Sandbox supervisor, budget, recovery | OS processes, Linux, containers, timeouts; demonstrate cancellation. |
| 4 | Model adapter and candidate loop | HTTP, structured output, prompt risks; compare scripted vs model proposals. |
| 5 | ECAR policy and graph | DAGs, priority queues, utility, uncertainty; implement policy without framework. |
| 6 | Pilot benchmark | Statistics, controls, randomization; calculate paired differences. |
| 7-8 | R1 preregistered held-out evaluation | Experimental design, leakage, bootstrap, research writing. |
| 9 | R2 web workspace | REST, React, state and accessible visualization. |
| 10 | Optional GPU/remote worker | Profiling, CUDA concepts, networking and distributed leases. |
| 11 | SDK, documentation, open-source release | Packaging, code review, issue triage and licenses. |
| 12 | Research submission / portfolio / user interviews | Explain results, limitations, demo and technical interview. |

## First two weeks (after exams, suggested 1-2 hours/day)
Days 1-2: rustup, Cargo, Git, variables and functions. Days 3-4: ownership, borrowing, structs and enums. Days 5-6: Result, Vec, iterators and unit tests; implement accuracy over a fixed prediction fixture. Day 7: recap and explain leakage. Days 8-9: serde JSON, hashes and typed manifests. Days 10-11: CLI baseline and repeat run. Days 12-13: reviewed Python digits worker contract and Rust evaluator (or defer worker until Rust fixture passes). Day 14: fresh replay, oral quiz and report; proceed only if you can explain every file. Begin with reviewed code; no generated autonomous code execution yet.

## Learning loop
For each module: conceptual lesson -> small manual implementation -> user attempt -> test -> code review -> explain tradeoffs -> add one feature. Quiz topics: why a seed is insufficient alone; why hidden data is separate; why atomic reservations matter; why a model verdict is not evidence; how to measure improvement fairly. Do not ask a coding agent to implement the entire repository at once. Portfolio evidence: tagged releases, benchmark tables, architecture docs, issue history, demo and reproducibility instructions.
