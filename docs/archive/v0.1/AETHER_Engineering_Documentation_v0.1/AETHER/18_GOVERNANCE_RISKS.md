# 18 | Governance, risk register and decision gates

| Risk | Likelihood / impact (estimate) | Mitigation / trigger |
|---|---|---|
| Novelty overlaps recent work | High / high | Literature gate; narrow question or replication study. |
| Scope creep | High / high | CLI first; no UI, remote compute or SDK until R0 evidence. |
| Insufficient hardware | Medium / high | CPU tasks; measure RAM/storage; cloud opt-in. |
| Cloud credits unavailable | Medium / medium | Local-only benchmark and scripted proposals. |
| LLM inconsistency / cost | High / medium | Mock baseline, pinned model, caps, repeated runs. |
| Invalid scientific claims | High / high | External verifier, held-out data, replay, abstention. |
| Sandbox failure | Medium / critical | Reviewed templates; dedicated isolation; deny by default. |
| Study underpowered | High / medium | Pilot variance; power review; report uncertainty. |
| Licenses restrict release | Medium / high | License inventory; separate nonredistributable artifacts. |
| Developer overload / exams | High / medium | Weekly small milestone, learning-first schedule. |

## Ownership and approvals
Developer owns implementation and learns core modules; faculty mentor (not yet confirmed) reviews experimental design and claims; independent reviewer (not yet assigned) checks reproduction and release. User approves budgets, licenses, public release and model providers. No external deployment or submission is implied. Track decisions and rationale in version-controlled ADRs. Changes to metrics, caps, tasks or evaluator after registration create new study version, not silent edits.

## Stop / go criteria
Go R0 only after safe environment and public data license verified. Go R1 only after baseline replay, provenance and budget tests pass. Go main study only after prior-art review, pilot and frozen protocol. Go R2 remote only after isolation and credential review. Go public only after licenses, redaction, benchmark limitations and security review. If any gate fails, document failure and reduce scope; do not compensate with unverified claims.
