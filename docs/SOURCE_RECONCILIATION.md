# Source reconciliation

Rust v0.2 controls. Its PDF, Markdown, diagrams and schema drafts are archived unchanged. The following readings make inconsistencies visible for owner design review.

| Source tension | Genesis interpretation or pending decision |
|---|---|
| v0.2 §01/11 give a scikit-learn digits baseline; §21 and Genesis §22 require a Python-free fixture first | Phase 1 built-in synthetic classification, followed by wider R0 reviewed digits work after exact data rights and isolation are verified. |
| §13 lists `verify <claim-id>`, approve/run and export; §21 lists `verify <run-id>` and report | The first contract uses run ID for replay and campaign ID for report. Richer claim validation and export follow later. |
| §21 describes the native fixture using the same contract as reviewed workers | Phase 1 uses fixed in-process input and typed results. Propagate protocol identity and manifest provenance. File-envelope conformance enters with the worker boundary. |
| §06 describes SUPPORTED before verdicts; source claim diagram permits direct REJECTED or INCONCLUSIVE | VERIFIED always requires SUPPORTED and independent checks. Immediate negative outcomes require trusted contradictory evidence or a documented lack of requisite evidence. |
| §08 dependency relation requires claim endpoints, while draft evidence edge can point to metric or artifact only | Typed claim dependencies use a separate acyclic edge set, and typed evidence edges link measurements or blobs. |
| Source run ID, attempt ID and lease are used in different contexts | Treat logical run, execution attempt and lease fence as distinct. Phase 1 run IDs identify fixture attempts. Reattempts later use distinct IDs and accounted cost. |
| Draft protocol example includes placeholder digests and mixed units; worker and proposal/event versions differ | Keep archive example non-executable. Working schemas/examples use defined integer units and synthetic digest shape. Version each contract independently. |
| §05 allows specialized reviewed Python verification; §21 makes Rust authoritative | External metric computation might be reviewed later. Rust binds pinned evidence and records authoritative transitions. |
| §17 indicative timeline places model proposals before ECAR policy; Genesis phases put ECAR first | Develop deterministic policy and baselines with scripted actions, then approved model proposals. Keep provider choice and reference models open. |
| §19 general wording calls ADRs proposed, with one explicit accepted Rust ADR | Rust-first is accepted. Other defaults and refinements are proposals until the owner records approval. |
| Handoff start guide refers to a v0.1 ZIP | Current handoff includes expanded v0.1 copies and its PDF. No local v0.1 ZIP was found in the searched handoff and Downloads. |
| v0.2 prior art is preliminary | New primary-source findings constrain originality. Benchmark and efficacy remain hypotheses. [Originality review](research/ORIGINALITY_ASSESSMENT.md) |

See the [archive index](archive/README.md), [native fixture scope](roadmap/MILESTONE_01_NATIVE_FIXTURE.md) and [decision log](governance/DECISION_LOG.md).
