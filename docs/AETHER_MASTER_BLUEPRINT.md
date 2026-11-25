# AETHER master blueprint

**Design review · 24 November 2026 · Phase 0**

AETHER is a local-first computational research laboratory. Build a deterministic experiment loop first. Add evidence and supervised execution, then test an adaptive policy where the same experiment can produce a verified result, a rejection or an inconclusive outcome.

## Review in this order

| Review topic | Start here | Decision or outcome |
|---|---|---|
| Original brief | [Archive and inventory](archive/README.md) | Rust v0.2 controls. Source and working specifications are traceable. |
| Product | [PRD](product/PRODUCT_REQUIREMENTS.md) and [scope](product/MVP_SCOPE.md) | Single-user CPU baseline before expansion. |
| Native system | [Architecture](architecture/README.md) and [engineering](engineering/README.md) | Pure Rust core, thin CLI, adapter boundaries and explicit local state. |
| Evidence and contracts | [Worker and event contracts](../contracts/README.md) and [diagrams](diagrams/README.md) | Scoped execution, exact provenance, auditable verdicts and typed faults. |
| Scientific method | [ECAR](research/ECAR_SPECIFICATION.md), [prior art](research/COMPETITOR_ANALYSIS.md) and [originality](research/ORIGINALITY_ASSESSMENT.md) | Frozen, matched comparisons distinguish adaptive allocation and graph effects. |
| Runtime risk | [Threat model](security/THREAT_MODEL.md) and [risks](governance/RISK_REGISTER.md) | Reviewed tasks and independent evidence. Linux-first qualification before external workers. |
| Delivery | [Roadmap](roadmap/MASTER_ROADMAP.md), [environment](operations/DEVELOPMENT_ENVIRONMENT.md) and [traceability](testing/REQUIREMENTS_TRACEABILITY.md) | Evidence-based phase gates. |
| Learning and career | [Lesson 1](learning/LESSON_01_CLASSIFICATION_AND_RUST.md) and [career alignment](career/CAREER_ALIGNMENT.md) | Explain the evaluator by hand and build demonstrable engineering artifacts. |

## Architecture and study in one view

The owner approves immutable protocol and policy. The Rust controller checks admissibility, reserves resource caps, dispatches reviewed tasks, ingests bounded outputs and applies trusted evaluation. SQLite and immutable blobs eventually retain projections, evidence, usage and audit history. ECAR ranks admissible exploration, replay and challenge actions or stops. An independent verifier applies the frozen protocol to a claim.

The proposed study compares dependency-aware adaptive verification with fixed and flat alternatives under matched caps and the same evaluator. A close prior method already uses claim dependencies and cost-aware verification. AETHER's empirical question concerns the end-to-end action mix within an auditable experiment loop. Effect size, originality and performance are open research questions.

## Approval checkpoint

The owner has authorized implementation-first development. The native fixture and local workbench are available through the [launch guide](operations/LOCAL_WORKBENCH.md). Review the [first milestone](roadmap/MILESTONE_01_NATIVE_FIXTURE.md), [source reconciliation](SOURCE_RECONCILIATION.md), [documentation audit](DOCUMENTATION_AUDIT.md) and [current status](../PROJECT_STATUS.md) for the implemented scope and remaining gates.
