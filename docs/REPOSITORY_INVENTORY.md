# Repository inventory

Project root is the requested AETHER directory. Git metadata, generated outputs and pre-existing local source staging trees are outside this inventory. Original package files are listed by digest and path in the [archive manifest](archive/SHA256SUMS).

## Complete directory tree

```text
AETHER/
├── .github/
│   └── workflows/
├── benchmarks/
├── contracts/
│   ├── examples/
│   └── schemas/
├── crates/
│   ├── aether-api/
│   │   ├── src/
│   │   └── tests/
│   ├── aether-cli/
│   │   ├── src/
│   │   └── tests/
│   ├── aether-core/
│   │   └── src/
│   ├── aether-engine/
│   │   └── src/
│   ├── aether-runner/
│   │   └── src/
│   └── aether-store/
│       └── src/
├── docs/
│   ├── architecture/
│   ├── archive/
│   │   ├── genesis/
│   │   ├── handoff/
│   │   ├── v0.1/
│   │   │   ├── AETHER/
│   │   │   │   ├── contracts/
│   │   │   │   └── diagrams/
│   │   │   └── AETHER_Engineering_Documentation_v0.1/
│   │   │       └── AETHER/
│   │   │           ├── contracts/
│   │   │           └── diagrams/
│   │   └── v0.2/
│   │       └── AETHER/
│   │           ├── contracts/
│   │           └── diagrams/
│   ├── career/
│   ├── decisions/
│   ├── design/
│   ├── diagrams/
│   │   └── rendered/
│   ├── engineering/
│   ├── governance/
│   ├── learning/
│   ├── operations/
│   ├── product/
│   ├── research/
│   ├── roadmap/
│   ├── security/
│   └── testing/
├── infra/
├── learning/
│   └── src/
│       └── bin/
├── research/
│   ├── benchmarks/
│   ├── experiments/
│   ├── hypotheses/
│   ├── literature/
│   ├── protocols/
│   ├── publications/
│   └── results/
├── scripts/
├── taskpacks/
├── tests/
├── web/
│   ├── src/
│   └── tests/
└── workers/
    └── python/
```

## Files by directory

395 repository files including 102 archived sources. The list below links the working files and archive inventory.

### Root

- [.editorconfig](../.editorconfig)
- [.gitattributes](../.gitattributes)
- [.gitignore](../.gitignore)
- [AGENTS.md](../AGENTS.md)
- [Cargo.lock](../Cargo.lock)
- [Cargo.toml](../Cargo.toml)
- [CHANGELOG.md](../CHANGELOG.md)
- [CITATION.cff](../CITATION.cff)
- [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)
- [CODEBASE_TOUR.md](../CODEBASE_TOUR.md)
- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [LICENSE_DECISION.md](../LICENSE_DECISION.md)
- [NEXT_ACTIONS.md](../NEXT_ACTIONS.md)
- [PROJECT_CONTEXT.md](../PROJECT_CONTEXT.md)
- [PROJECT_STATUS.md](../PROJECT_STATUS.md)
- [README.md](../README.md)
- [SECURITY.md](../SECURITY.md)

### .github

- [README.md](../.github/README.md)

### .github/workflows

- [ci.yml](../.github/workflows/ci.yml)

### benchmarks

- [README.md](../benchmarks/README.md)

### contracts

- [README.md](../contracts/README.md)

### contracts/examples

- [claim.example.json](../contracts/examples/claim.example.json)
- [event.example.json](../contracts/examples/event.example.json)
- [protocol.example.json](../contracts/examples/protocol.example.json)
- [worker-job.example.json](../contracts/examples/worker-job.example.json)
- [worker-result.example.json](../contracts/examples/worker-result.example.json)

### contracts/schemas

- [claim.schema.json](../contracts/schemas/claim.schema.json)
- [event.schema.json](../contracts/schemas/event.schema.json)
- [protocol.schema.json](../contracts/schemas/protocol.schema.json)
- [worker-job.schema.json](../contracts/schemas/worker-job.schema.json)
- [worker-result.schema.json](../contracts/schemas/worker-result.schema.json)

### crates/aether-api

- [Cargo.toml](../crates/aether-api/Cargo.toml)
- [README.md](../crates/aether-api/README.md)

### crates/aether-api/src

- [http.rs](../crates/aether-api/src/http.rs)
- [lab.rs](../crates/aether-api/src/lab.rs)
- [literature.rs](../crates/aether-api/src/literature.rs)
- [main.rs](../crates/aether-api/src/main.rs)
- [planning.rs](../crates/aether-api/src/planning.rs)

### crates/aether-api/tests

- [platform.rs](../crates/aether-api/tests/platform.rs)

### crates/aether-cli

- [Cargo.toml](../crates/aether-cli/Cargo.toml)
- [README.md](../crates/aether-cli/README.md)

### crates/aether-cli/src

- [main.rs](../crates/aether-cli/src/main.rs)
- [native.rs](../crates/aether-cli/src/native.rs)

### crates/aether-cli/tests

- [native_fixture.rs](../crates/aether-cli/tests/native_fixture.rs)

### crates/aether-core

- [Cargo.toml](../crates/aether-core/Cargo.toml)
- [README.md](../crates/aether-core/README.md)

### crates/aether-core/src

- [lib.rs](../crates/aether-core/src/lib.rs)

### crates/aether-engine

- [Cargo.toml](../crates/aether-engine/Cargo.toml)
- [README.md](../crates/aether-engine/README.md)

### crates/aether-engine/src

- [budget.rs](../crates/aether-engine/src/budget.rs)
- [lib.rs](../crates/aether-engine/src/lib.rs)
- [scheduler.rs](../crates/aether-engine/src/scheduler.rs)
- [simulation.rs](../crates/aether-engine/src/simulation.rs)

### crates/aether-runner

- [Cargo.toml](../crates/aether-runner/Cargo.toml)
- [README.md](../crates/aether-runner/README.md)

### crates/aether-runner/src

- [lib.rs](../crates/aether-runner/src/lib.rs)

### crates/aether-store

- [Cargo.toml](../crates/aether-store/Cargo.toml)
- [README.md](../crates/aether-store/README.md)

### crates/aether-store/src

- [001_initial.sql](../crates/aether-store/src/001_initial.sql)
- [evidence.rs](../crates/aether-store/src/evidence.rs)
- [execution.rs](../crates/aether-store/src/execution.rs)
- [lib.rs](../crates/aether-store/src/lib.rs)
- [model.rs](../crates/aether-store/src/model.rs)
- [read.rs](../crates/aether-store/src/read.rs)
- [tests.rs](../crates/aether-store/src/tests.rs)

### docs

- [AETHER_MASTER_BLUEPRINT.md](AETHER_MASTER_BLUEPRINT.md)
- [DOCUMENTATION_AUDIT.md](DOCUMENTATION_AUDIT.md)
- [index.md](index.md)
- [REPOSITORY_INVENTORY.md](REPOSITORY_INVENTORY.md)
- [SOURCE_RECONCILIATION.md](SOURCE_RECONCILIATION.md)

### docs/architecture

- [API_DESIGN.md](architecture/API_DESIGN.md)
- [COMPONENT_ARCHITECTURE.md](architecture/COMPONENT_ARCHITECTURE.md)
- [DATA_ARCHITECTURE.md](architecture/DATA_ARCHITECTURE.md)
- [DATABASE_DESIGN.md](architecture/DATABASE_DESIGN.md)
- [DEPLOYMENT_ARCHITECTURE.md](architecture/DEPLOYMENT_ARCHITECTURE.md)
- [EVENT_ARCHITECTURE.md](architecture/EVENT_ARCHITECTURE.md)
- [FAILURE_AND_RECOVERY.md](architecture/FAILURE_AND_RECOVERY.md)
- [HARDWARE_ARCHITECTURE.md](architecture/HARDWARE_ARCHITECTURE.md)
- [HIGH_LEVEL_DESIGN.md](architecture/HIGH_LEVEL_DESIGN.md)
- [LOW_LEVEL_DESIGN.md](architecture/LOW_LEVEL_DESIGN.md)
- [README.md](architecture/README.md)
- [RESOURCE_MANAGEMENT.md](architecture/RESOURCE_MANAGEMENT.md)
- [SCALABILITY_DESIGN.md](architecture/SCALABILITY_DESIGN.md)
- [SOFTWARE_ARCHITECTURE.md](architecture/SOFTWARE_ARCHITECTURE.md)
- [STATE_MACHINES.md](architecture/STATE_MACHINES.md)
- [SYSTEM_ARCHITECTURE.md](architecture/SYSTEM_ARCHITECTURE.md)
- [WORKER_PROTOCOL.md](architecture/WORKER_PROTOCOL.md)

### docs/archive

- [README.md](archive/README.md)
- [SHA256SUMS](archive/SHA256SUMS)

### docs/career

- [CAREER_ALIGNMENT.md](career/CAREER_ALIGNMENT.md)
- [INTERNSHIP_PREPARATION.md](career/INTERNSHIP_PREPARATION.md)
- [OPEN_SOURCE_STRATEGY.md](career/OPEN_SOURCE_STRATEGY.md)
- [PORTFOLIO_STRATEGY.md](career/PORTFOLIO_STRATEGY.md)

### docs/decisions

- [ADR-001.md](decisions/ADR-001.md)
- [ADR-002.md](decisions/ADR-002.md)
- [ADR-003.md](decisions/ADR-003.md)
- [ADR-004.md](decisions/ADR-004.md)
- [ADR-005.md](decisions/ADR-005.md)
- [ADR-006.md](decisions/ADR-006.md)
- [ADR-007.md](decisions/ADR-007.md)
- [ADR-008.md](decisions/ADR-008.md)
- [ADR-009.md](decisions/ADR-009.md)
- [ADR-010.md](decisions/ADR-010.md)
- [ADR-011.md](decisions/ADR-011.md)

### docs/design

- [CLI_AND_RESEARCH_JOURNEY.md](design/CLI_AND_RESEARCH_JOURNEY.md)
- [README.md](design/README.md)

### docs/diagrams

- [budget-ledger.mmd](diagrams/budget-ledger.mmd)
- [campaign-state.mmd](diagrams/campaign-state.mmd)
- [ci-cd.mmd](diagrams/ci-cd.mmd)
- [claim-state.mmd](diagrams/claim-state.mmd)
- [components.mmd](diagrams/components.mmd)
- [containers.mmd](diagrams/containers.mmd)
- [crate-dependencies.mmd](diagrams/crate-dependencies.mmd)
- [deployment.mmd](diagrams/deployment.mmd)
- [distributed-workers.mmd](diagrams/distributed-workers.mmd)
- [ecar-lifecycle.mmd](diagrams/ecar-lifecycle.mmd)
- [entity-relationships.mmd](diagrams/entity-relationships.mmd)
- [evidence-graph.mmd](diagrams/evidence-graph.mmd)
- [experiment-execution.mmd](diagrams/experiment-execution.mmd)
- [failure-recovery.mmd](diagrams/failure-recovery.mmd)
- [README.md](diagrams/README.md)
- [run-lifecycle.mmd](diagrams/run-lifecycle.mmd)
- [sandbox.mmd](diagrams/sandbox.mmd)
- [system-context.mmd](diagrams/system-context.mmd)
- [trust-boundaries.mmd](diagrams/trust-boundaries.mmd)
- [user-journey.mmd](diagrams/user-journey.mmd)
- [verification-pipeline.mmd](diagrams/verification-pipeline.mmd)
- [worker-communication.mmd](diagrams/worker-communication.mmd)

### docs/diagrams/rendered

- [budget-ledger.svg](diagrams/rendered/budget-ledger.svg)
- [campaign-state.svg](diagrams/rendered/campaign-state.svg)
- [ci-cd.svg](diagrams/rendered/ci-cd.svg)
- [claim-state.svg](diagrams/rendered/claim-state.svg)
- [components.svg](diagrams/rendered/components.svg)
- [containers.svg](diagrams/rendered/containers.svg)
- [crate-dependencies.svg](diagrams/rendered/crate-dependencies.svg)
- [deployment.svg](diagrams/rendered/deployment.svg)
- [distributed-workers.svg](diagrams/rendered/distributed-workers.svg)
- [ecar-lifecycle.svg](diagrams/rendered/ecar-lifecycle.svg)
- [entity-relationships.svg](diagrams/rendered/entity-relationships.svg)
- [evidence-graph.svg](diagrams/rendered/evidence-graph.svg)
- [experiment-execution.svg](diagrams/rendered/experiment-execution.svg)
- [failure-recovery.svg](diagrams/rendered/failure-recovery.svg)
- [run-lifecycle.svg](diagrams/rendered/run-lifecycle.svg)
- [sandbox.svg](diagrams/rendered/sandbox.svg)
- [system-context.svg](diagrams/rendered/system-context.svg)
- [trust-boundaries.svg](diagrams/rendered/trust-boundaries.svg)
- [user-journey.svg](diagrams/rendered/user-journey.svg)
- [verification-pipeline.svg](diagrams/rendered/verification-pipeline.svg)
- [worker-communication.svg](diagrams/rendered/worker-communication.svg)

### docs/engineering

- [CODING_STANDARDS.md](engineering/CODING_STANDARDS.md)
- [CONCURRENCY_MODEL.md](engineering/CONCURRENCY_MODEL.md)
- [DEPENDENCY_POLICY.md](engineering/DEPENDENCY_POLICY.md)
- [ERROR_HANDLING.md](engineering/ERROR_HANDLING.md)
- [MEMORY_MANAGEMENT.md](engineering/MEMORY_MANAGEMENT.md)
- [PERFORMANCE_ENGINEERING.md](engineering/PERFORMANCE_ENGINEERING.md)
- [README.md](engineering/README.md)
- [REPOSITORY_STRUCTURE.md](engineering/REPOSITORY_STRUCTURE.md)
- [RUST_ENGINEERING_STANDARD.md](engineering/RUST_ENGINEERING_STANDARD.md)
- [SOFTWARE_REQUIREMENTS_SPECIFICATION.md](engineering/SOFTWARE_REQUIREMENTS_SPECIFICATION.md)
- [TECHNICAL_REQUIREMENTS.md](engineering/TECHNICAL_REQUIREMENTS.md)
- [TECHNOLOGY_STACK.md](engineering/TECHNOLOGY_STACK.md)

### docs/governance

- [ARCHITECTURE_DECISION_RECORDS.md](governance/ARCHITECTURE_DECISION_RECORDS.md)
- [DECISION_LOG.md](governance/DECISION_LOG.md)
- [DEPENDENCY_INVENTORY.md](governance/DEPENDENCY_INVENTORY.md)
- [DEVELOPMENT_POLICY.md](governance/DEVELOPMENT_POLICY.md)
- [HUMAN_APPROVAL_POLICY.md](governance/HUMAN_APPROVAL_POLICY.md)
- [LICENSE_EVALUATION.md](governance/LICENSE_EVALUATION.md)
- [RESEARCH_INTEGRITY_POLICY.md](governance/RESEARCH_INTEGRITY_POLICY.md)
- [RISK_REGISTER.md](governance/RISK_REGISTER.md)

### docs/learning

- [AI_RESEARCH_CURRICULUM.md](learning/AI_RESEARCH_CURRICULUM.md)
- [BEGINNER_LEARNING_PATH.md](learning/BEGINNER_LEARNING_PATH.md)
- [COMPUTER_SCIENCE_FOUNDATIONS.md](learning/COMPUTER_SCIENCE_FOUNDATIONS.md)
- [ENGINEERING_EXERCISES.md](learning/ENGINEERING_EXERCISES.md)
- [IMPLEMENTATION_API_AGENTS.md](learning/IMPLEMENTATION_API_AGENTS.md)
- [IMPLEMENTATION_COMPUTE_DISTRIBUTED.md](learning/IMPLEMENTATION_COMPUTE_DISTRIBUTED.md)
- [IMPLEMENTATION_ECAR_STATISTICS.md](learning/IMPLEMENTATION_ECAR_STATISTICS.md)
- [IMPLEMENTATION_OS_SECURITY.md](learning/IMPLEMENTATION_OS_SECURITY.md)
- [IMPLEMENTATION_RUST_EVALUATOR.md](learning/IMPLEMENTATION_RUST_EVALUATOR.md)
- [IMPLEMENTATION_STORAGE_GRAPHS.md](learning/IMPLEMENTATION_STORAGE_GRAPHS.md)
- [LESSON_01_CLASSIFICATION_AND_RUST.md](learning/LESSON_01_CLASSIFICATION_AND_RUST.md)
- [MATHEMATICS_FOR_AETHER.md](learning/MATHEMATICS_FOR_AETHER.md)
- [REBUILD_AETHER_FROM_SCRATCH.md](learning/REBUILD_AETHER_FROM_SCRATCH.md)
- [RUST_ZERO_TO_ADVANCED.md](learning/RUST_ZERO_TO_ADVANCED.md)
- [TECHNICAL_ASSESSMENTS.md](learning/TECHNICAL_ASSESSMENTS.md)
- [WEEKLY_LEARNING_PLAN.md](learning/WEEKLY_LEARNING_PLAN.md)

### docs/operations

- [CI_CD_DESIGN.md](operations/CI_CD_DESIGN.md)
- [COST_MANAGEMENT.md](operations/COST_MANAGEMENT.md)
- [DEVELOPMENT_ENVIRONMENT.md](operations/DEVELOPMENT_ENVIRONMENT.md)
- [DEVOPS_STRATEGY.md](operations/DEVOPS_STRATEGY.md)
- [DISASTER_RECOVERY.md](operations/DISASTER_RECOVERY.md)
- [LOCAL_WORKBENCH.md](operations/LOCAL_WORKBENCH.md)
- [OBSERVABILITY.md](operations/OBSERVABILITY.md)
- [RDLC.md](operations/RDLC.md)
- [RELEASE_MANAGEMENT.md](operations/RELEASE_MANAGEMENT.md)
- [SDLC.md](operations/SDLC.md)

### docs/product

- [COMPUTER_APPLICATIONS.md](product/COMPUTER_APPLICATIONS.md)
- [FUNCTIONAL_REQUIREMENTS.md](product/FUNCTIONAL_REQUIREMENTS.md)
- [MVP_SCOPE.md](product/MVP_SCOPE.md)
- [NON_FUNCTIONAL_REQUIREMENTS.md](product/NON_FUNCTIONAL_REQUIREMENTS.md)
- [PRODUCT_REQUIREMENTS.md](product/PRODUCT_REQUIREMENTS.md)
- [PRODUCT_VISION.md](product/PRODUCT_VISION.md)
- [PROJECT_CHARTER.md](product/PROJECT_CHARTER.md)
- [SUCCESS_METRICS.md](product/SUCCESS_METRICS.md)
- [USE_CASES.md](product/USE_CASES.md)
- [USER_PERSONAS.md](product/USER_PERSONAS.md)

### docs/research

- [ABLATION_STUDY_DESIGN.md](research/ABLATION_STUDY_DESIGN.md)
- [BASELINE_COMPARISON.md](research/BASELINE_COMPARISON.md)
- [BENCHMARK_RESULTS.md](research/BENCHMARK_RESULTS.md)
- [BENCHMARK_SPECIFICATION.md](research/BENCHMARK_SPECIFICATION.md)
- [COMPETITOR_ANALYSIS.md](research/COMPETITOR_ANALYSIS.md)
- [ECAR_SPECIFICATION.md](research/ECAR_SPECIFICATION.md)
- [EVALUATION_METRICS.md](research/EVALUATION_METRICS.md)
- [EXPERIMENTAL_METHODOLOGY.md](research/EXPERIMENTAL_METHODOLOGY.md)
- [LITERATURE_REVIEW.md](research/LITERATURE_REVIEW.md)
- [MATHEMATICAL_FOUNDATIONS.md](research/MATHEMATICAL_FOUNDATIONS.md)
- [ORIGINALITY_ASSESSMENT.md](research/ORIGINALITY_ASSESSMENT.md)
- [README.md](research/README.md)
- [REFERENCES.md](research/REFERENCES.md)
- [RELATED_WORK.md](research/RELATED_WORK.md)
- [REPRODUCIBILITY_PROTOCOL.md](research/REPRODUCIBILITY_PROTOCOL.md)
- [RESEARCH_ETHICS.md](research/RESEARCH_ETHICS.md)
- [RESEARCH_GAPS.md](research/RESEARCH_GAPS.md)
- [RESEARCH_HYPOTHESES.md](research/RESEARCH_HYPOTHESES.md)
- [RESEARCH_PUBLICATION_ROADMAP.md](research/RESEARCH_PUBLICATION_ROADMAP.md)
- [RESEARCH_VISION.md](research/RESEARCH_VISION.md)
- [STATISTICAL_ANALYSIS_PLAN.md](research/STATISTICAL_ANALYSIS_PLAN.md)

### docs/roadmap

- [IMPLEMENTATION_BACKLOG.md](roadmap/IMPLEMENTATION_BACKLOG.md)
- [MASTER_ROADMAP.md](roadmap/MASTER_ROADMAP.md)
- [MILESTONE_01_NATIVE_FIXTURE.md](roadmap/MILESTONE_01_NATIVE_FIXTURE.md)
- [MILESTONES.md](roadmap/MILESTONES.md)
- [PHASE_ZERO.md](roadmap/PHASE_ZERO.md)
- [RELEASE_CRITERIA.md](roadmap/RELEASE_CRITERIA.md)

### docs/security

- [AI_AGENT_SAFETY.md](security/AI_AGENT_SAFETY.md)
- [DATA_PRIVACY.md](security/DATA_PRIVACY.md)
- [INCIDENT_RESPONSE.md](security/INCIDENT_RESPONSE.md)
- [PERMISSION_MODEL.md](security/PERMISSION_MODEL.md)
- [PROMPT_INJECTION_DEFENSE.md](security/PROMPT_INJECTION_DEFENSE.md)
- [SANDBOX_DESIGN.md](security/SANDBOX_DESIGN.md)
- [SECRETS_MANAGEMENT.md](security/SECRETS_MANAGEMENT.md)
- [SECURITY_ARCHITECTURE.md](security/SECURITY_ARCHITECTURE.md)
- [SECURITY_REVIEW.md](security/SECURITY_REVIEW.md)
- [SUPPLY_CHAIN_SECURITY.md](security/SUPPLY_CHAIN_SECURITY.md)
- [THREAT_MODEL.md](security/THREAT_MODEL.md)

### docs/testing

- [ACCEPTANCE_CRITERIA.md](testing/ACCEPTANCE_CRITERIA.md)
- [BENCHMARK_VALIDATION.md](testing/BENCHMARK_VALIDATION.md)
- [FAILURE_INJECTION.md](testing/FAILURE_INJECTION.md)
- [PERFORMANCE_TESTING.md](testing/PERFORMANCE_TESTING.md)
- [QUALITY_GATES.md](testing/QUALITY_GATES.md)
- [REQUIREMENTS_TRACEABILITY.md](testing/REQUIREMENTS_TRACEABILITY.md)
- [TEST_PLAN.md](testing/TEST_PLAN.md)
- [TEST_RESULTS.md](testing/TEST_RESULTS.md)
- [TEST_STRATEGY.md](testing/TEST_STRATEGY.md)

### infra

- [README.md](../infra/README.md)

### learning

- [Cargo.lock](../learning/Cargo.lock)
- [Cargo.toml](../learning/Cargo.toml)
- [README.md](../learning/README.md)

### learning/src/bin

- [lesson_01.rs](../learning/src/bin/lesson_01.rs)

### research

- [README.md](../research/README.md)

### research/benchmarks

- [README.md](../research/benchmarks/README.md)

### research/experiments

- [README.md](../research/experiments/README.md)

### research/hypotheses

- [README.md](../research/hypotheses/README.md)

### research/literature

- [README.md](../research/literature/README.md)

### research/protocols

- [README.md](../research/protocols/README.md)

### research/publications

- [README.md](../research/publications/README.md)

### research/results

- [ecar-policy-simulation.json](../research/results/ecar-policy-simulation.json)
- [local-sort-campaign.json](../research/results/local-sort-campaign.json)
- [native-sort-benchmark.json](../research/results/native-sort-benchmark.json)
- [README.md](../research/results/README.md)

### scripts

- [README.md](../scripts/README.md)
- [start-local.ps1](../scripts/start-local.ps1)
- [start-local.sh](../scripts/start-local.sh)
- [validate_docs.py](../scripts/validate_docs.py)

### taskpacks

- [example-campaign.json](../taskpacks/example-campaign.json)
- [example-planning.json](../taskpacks/example-planning.json)
- [README.md](../taskpacks/README.md)
- [seeded-sort-v1.json](../taskpacks/seeded-sort-v1.json)

### tests

- [browser-smoke.cjs](../tests/browser-smoke.cjs)
- [README.md](../tests/README.md)

### web

- [index.html](../web/index.html)
- [package-lock.json](../web/package-lock.json)
- [package.json](../web/package.json)
- [README.md](../web/README.md)
- [tsconfig.json](../web/tsconfig.json)
- [vite.config.ts](../web/vite.config.ts)

### web/src

- [api.ts](../web/src/api.ts)
- [App.tsx](../web/src/App.tsx)
- [campaignViews.tsx](../web/src/campaignViews.tsx)
- [components.tsx](../web/src/components.tsx)
- [consoleViews.tsx](../web/src/consoleViews.tsx)
- [main.tsx](../web/src/main.tsx)
- [styles.css](../web/src/styles.css)

### web/tests

- [api.test.mjs](../web/tests/api.test.mjs)

### workers/python

- [README.md](../workers/python/README.md)
