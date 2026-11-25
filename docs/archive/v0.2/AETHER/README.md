# AETHER | Research & Engineering Design Authority
Version 0.2.0 · 24 November 2026 · Proposed / pre-implementation

AETHER is a human-governed computational research laboratory. Its first research hypothesis is that claim-level evidence requirements and adaptive verification allocation improve verified scientific utility under a fixed compute budget. It is not an autonomous frontier model trainer or unrestricted self-improving system.

## Reading order
- 01 Charter & scope; 02 Prior art & novelty; 03 PRD; 04 TRD/SRS; 05 System architecture; 06 Detailed system design; 07 Technology stack; 08 Data design; 09 API and events; 10 Research methodology; 11 Benchmark & evaluation; 12 Security & responsible research; 13 UX and computer applications; 14 SDLC/RDLC; 15 Testing & quality; 16 DevOps & operations; 17 Roadmap & learning; 18 Governance & risks; 19 ADRs and traceability; 20 References.
- `contracts/` contains illustrative machine-readable interface drafts; `diagrams/` contains Mermaid architecture sources. No runnable application is delivered.

## Design authority
Requirements use PRD-F, TRD-N and SEC IDs; acceptance gates must be demonstrated by tests. The research protocol overrides feature ambitions where experimental validity conflicts. This is a proposal, not a claim of completed implementation, novelty, or measured performance. Defaults are hypotheses to review, including time budgets and sample size. Phase 0 must verify dependency versions, hardware RAM/storage, licenses, and cloud availability before installing anything.

## Rust-first revision
ADR-009 is the controlling language decision: native Rust for all trusted control-plane logic; isolated Python only for approved ML task workers. Read 21_RUST_ENGINEERING.md before implementation. The 0.1 design is superseded on language, build and runtime choices. Research hypotheses, budgets and security gates are unchanged. No code is implemented.
