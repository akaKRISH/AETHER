# 01 | Project charter and scope

## Mission
Build an open-source laboratory for constrained computational discovery: formulate candidate hypotheses, execute experiments, attach evidence to individual claims, challenge conclusions, and reproduce results. Long-term outcomes: an internship-quality portfolio, research submission, useful open-source project and potential startup. Priority order: scientific validity, safety, learning, then breadth and presentation.

## Initial release (R0)
Single local user, fixed benchmark tasks, CPU-first supervised runs, one worker, deterministic evaluation, append-only evidence ledger, immutable artifact manifests, command-line interface (CLI). First task: compare a scikit-learn baseline against candidate classifiers on a pinned public dataset; use a hidden test split, fixed seeds and a predefined tolerance. R0 supports one explicit plan with human approval, not autonomous unrestricted execution.

## Research release (R1)
Bounded candidate generation, claim extraction into typed assertions, evidence graph, budget scheduler, held-out validation, independently executed replication and report export. Compare with baseline agent configurations at equal resources. R2 adds algorithm evolution, optional GPU profiling, a web interface and a remote worker; R3 explores extension SDK and multi-user service only after threat review.

## Users and boundaries
Primary user: undergraduate developer learning each subsystem. Secondary: computational researcher who wants auditable results; evaluator examining reproducibility; contributor adding a task adapter. No wet-lab automation, personal data, medical recommendations, cyber exploitation, uncontrolled internet, autonomous publication, model self-modification, secret discovery or unsupervised production deployment. Models cannot edit the orchestrator, evaluator or policy. Humans approve resource budgets, data, tools and release decisions.

## Success definitions
R0: reproduce a frozen baseline from a fresh environment; report all trials including failures. R1: complete preregistered evaluation and publish measured results with uncertainty, even if the hypothesis fails. Portfolio: explain each core component, show tests, a demo, an architecture decision record and experiment artifacts. Open source: documented setup, contributor guide and licensed task pack. Startup: interview at least five potential users after R1; no assumption of market demand.

## Language boundary
Rust owns the research platform, scheduler and authoritative results; Python is a replaceable experiment workload, not a second controller. Originality refers to ECAR research, not merely language choice.
