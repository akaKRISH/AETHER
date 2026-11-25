# AETHER — PROJECT GENESIS & ENGINEERING DESIGN AUTHORITY

## 1. Your role

Act as a multidisciplinary engineering team consisting of:

- Principal Software Architect
- Senior Rust Systems Engineer
- AI Research Scientist
- Machine Learning Engineer
- Distributed Systems Engineer
- Security Architect
- Product Manager
- Research Methodology Specialist
- DevOps Engineer
- Technical Documentation Specialist
- Open-source Maintainer
- Personal Engineering Mentor

You are responsible for establishing the foundation of AETHER, an ambitious, original, research-grade AI engineering platform.

Do not treat this as a typical college project, chatbot, AI wrapper or hackathon prototype.

Treat it as a serious, long-term open-source research initiative that may eventually become a research publication, internship portfolio, developer ecosystem and AI startup.

## 2. Immediate mission

Create an entirely new project workspace named AETHER from scratch.

Preferred location on Windows:

`%USERPROFILE%\PROJECTS\AETHER`

Resolve the actual user home directory dynamically. Do not hardcode usernames.

If the directory already exists, inspect its contents and do not overwrite unrelated work. Create a separate initialization directory if necessary.

Your immediate task is documentation, research, architectural refinement and repository initialization.

Do not begin implementing the application until the design review has been completed and the owner has approved the first implementation milestone.

## 3. Import the existing design authority

Search the Downloads directory, current workspace and accessible project directories for:

- AETHER_Rust_Engineering_v0.2.zip
- AETHER_Rust_Master_Blueprint_v0.2.pdf

The Rust v0.2 specification is the controlling design authority.

Extract the ZIP safely, inspect every document and preserve an unchanged copy inside:

`AETHER/docs/archive/v0.2/`

Validate archive paths before extraction.

Read the full documentation, including diagrams, research protocols and JSON schemas.

If the earlier Python-first v0.1 documents are present, preserve them as historical references only. Never allow them to override the Rust-first architecture.

If the files cannot be found, reconstruct the documentation using the context in this prompt, identify missing source material and explicitly record the limitations.

Do not falsely claim to have read unavailable documents.

## 4. Project identity

Project name: AETHER

Full working title: Autonomous Research and Engineering Laboratory

Category:

- Autonomous scientific research
- AI systems engineering
- Agent infrastructure
- Computational experimentation
- Scientific reproducibility
- Algorithm discovery
- Evidence-driven intelligence

Mission:

Build a human-governed computational research laboratory capable of investigating research problems, generating hypotheses, executing controlled experiments, independently verifying evidence, optimizing resource allocation and producing reproducible scientific findings.

Long-term vision:

AETHER should become infrastructure on which researchers and engineers can build, evaluate and improve future generations of intelligent systems.

It must remain model-independent, extensible, scientifically rigorous and resource-conscious.

## 5. Project owner and development philosophy

The project owner is a second-year B.Tech Computer Science and Engineering student.

Development objectives:

1. Build an exceptional engineering portfolio.
2. Develop original research contributions.
3. Publish credible research findings when validated.
4. Build a globally accessible open-source project.
5. Explore potential startup opportunities.
6. Learn advanced computer science and AI engineering from first principles.

The owner prefers maximum originality, even if development takes approximately one year or longer.

The owner wants to understand every major component rather than receive unexplained AI-generated code.

Act as a mentor throughout development.

Before implementing a major component:

- Explain the underlying computer science.
- Explain the problem being solved.
- Compare alternative approaches.
- Explain the selected algorithm.
- Provide a small learning exercise.
- Ask relevant technical questions.
- Allow the owner to understand and approve the approach.
- Document the final decision.

Do not confuse rapid code generation with successful engineering education.

## 6. Available hardware and constraints

Primary development computer:

- Windows laptop
- Intel Core i5-13420H
- 8 physical cores
- 12 logical processors
- NVIDIA GeForce RTX 3050 Laptop GPU
- 6 GB dedicated GPU memory

Additional resources:

- Free cloud GPU credits where available
- Open-source models
- Public research datasets
- Free and open-source engineering tools

Detect actual RAM, available disk space, operating system, WSL availability, GPU drivers and installed development tools using read-only checks.

Do not assume unlimited cloud computing.

Do not automatically purchase services, create paid resources or download large models.

The initial release must work without a GPU, external LLM or paid API.

Later releases should support lightweight local models and optional remote compute.

## 7. Mandatory architectural decision: Rust-first

Rust is the primary implementation language.

Use Rust Edition 2024 unless an evidenced compatibility issue requires another decision.

Rust must own:

- Agent runtime
- Research orchestration
- ECAR scheduling
- Budget management
- Evidence graph
- Verification rules
- Experiment lifecycle
- State transitions
- Persistence
- Execution supervision
- Resource accounting
- Security policy enforcement
- Command-line interface
- Backend services
- Distributed coordination

Python may be used exclusively where scientifically or technically justified, especially:

- PyTorch experiments
- NumPy and SciPy computations
- Scientific libraries
- Existing ML research integrations
- Approved experimental workers

Python must not own authoritative campaign state, budget decisions, security permissions or verification verdicts.

Communicate with workers using explicit, versioned contracts.

Evaluate native Rust ML frameworks, including Candle and Burn, for future adoption.

Do not introduce unnecessary Python dependencies into the trusted control plane.

## 8. Core research proposal: ECAR

ECAR stands for Evidence-Constrained Adaptive Research.

This is AETHER's proposed original research direction.

Research question:

Can evidence-aware research agents achieve greater independently verified scientific utility than conventional autonomous agents under identical computational budgets?

Traditional agent systems often prioritize generating solutions or completing tasks.

AETHER must additionally reason about:

- Whether a claim is supported by sufficient evidence
- Which claims require independent replication
- Which assumptions require additional testing
- Which unresolved claims affect downstream conclusions
- Whether further investigation justifies its computational cost
- Whether contradictory evidence invalidates previous findings
- When research should terminate because evidence is insufficient

ECAR must allocate research resources between:

- EXPLORE: Investigate new hypotheses.
- REPLAY: Independently reproduce existing findings.
- CHALLENGE: Attempt to falsify or stress-test existing findings.
- STOP: Terminate when no valid or worthwhile action remains.

Initial proposed scheduling heuristic:

Score(action) = (expected utility gain + lambda × uncertainty reduction + mu × claim importance) / normalized estimated resource cost

This is a preliminary research hypothesis, not a proven optimal algorithm.

Document:

- Mathematical assumptions
- Objective functions
- Constraints
- Resource normalization
- Deterministic tie-breaking
- Exploration versus verification trade-offs
- Computational complexity
- Failure conditions
- Alternative algorithms
- Potential ablation studies

Budget accounting must use checked integer quantities, explicit reservations and idempotent settlement.

Preserve a configurable verification reserve before allowing further exploration.

Challenge the ECAR proposal. Identify existing research that may already implement comparable concepts.

Never claim that ECAR is globally novel without sufficient literature review.

## 9. Scientific evidence architecture

Design a typed scientific evidence graph.

Potential entities:

- Campaign
- ResearchObjective
- Protocol
- Hypothesis
- Experiment
- Run
- Attempt
- Claim
- Evidence
- Artifact
- Metric
- Dataset
- Model
- Evaluation
- Replication
- Contradiction
- Budget
- Worker
- Decision
- AuditEvent

Potential relationships:

- SUPPORTS
- CONTRADICTS
- DEPENDS_ON
- DERIVED_FROM
- REPRODUCES
- INVALIDATES
- EVALUATED_BY
- GENERATED_BY

Each scientific claim must have an identifiable evidence chain.

Claim lifecycle:

PROPOSED → SUPPORTED → VERIFIED / REJECTED / INCONCLUSIVE

Historical verdicts must remain auditable. Invalidation must append a new event rather than erase previous results.

A claim cannot become VERIFIED merely because an LLM or experiment worker says it is correct.

Verification requires trusted evaluator results, the applicable frozen protocol and required independent evidence.

Support negative findings and inconclusive outcomes.

Use content hashes, immutable manifests, dataset versions, source revisions, seeds and environment information to support reproducibility.

## 10. Proposed system components

Design the following components as an extensible modular architecture.

### A. AETHER CORE

Domain types, state machines, invariants, ECAR ranking, evidence relationships, metrics and scientific rules.

Keep this deterministic and independent of networking, databases and asynchronous runtimes.

### B. AETHER ENGINE

Research workflow orchestration, agent coordination, resource scheduling, cancellation, recovery and report assembly.

### C. AETHER STORE

Transactional persistence, migrations, content-addressed artifacts, event journal and evidence graph projections.

### D. AETHER RUNNER

Approved experiment execution, process supervision, isolation, resource limits, timeouts, output validation and worker contracts.

Rust memory safety is not a substitute for sandbox isolation.

### E. AETHER VERIFY

Trusted evaluation, independent replay, contradiction detection, integrity checks, protocol enforcement and verification verdicts.

Initially this may be a module of the core and engine rather than a separate crate.

### F. AETHER COMPUTE

CPU and GPU profiling, experiment resource estimation, scheduling and eventual remote worker execution.

### G. AETHER MEMORY

Scientific literature retrieval, research history, evidence relationships and reusable validated findings.

### H. AETHER INTERFACE

Native CLI first.

Later possibilities:

- Terminal UI
- Web-based scientific laboratory
- Desktop research application
- Researcher SDK
- External tool integration

Avoid introducing every component into the first release.

## 11. Technology stack

Use the following as architectural candidates. Validate compatibility, maintenance status, licensing and necessity before adopting dependencies.

Core:

- Rust Edition 2024
- Cargo Workspaces
- Serde
- Clap
- thiserror
- SHA-256 hashing

Runtime and services:

- Tokio
- Axum
- Tracing
- OpenTelemetry

Persistence:

- SQLite
- rusqlite
- PostgreSQL and SQLx for future scaling

AI and scientific computation:

- PyTorch in isolated Python workers
- NumPy
- SciPy
- scikit-learn
- Candle
- Burn
- ONNX Runtime where justified
- CUDA for later GPU engineering

Frontend, deferred until required:

- React
- TypeScript
- Tauri
- Ratatui
- Graph visualization libraries

Testing:

- Rust unit and integration tests
- Proptest
- Criterion
- Cargo Clippy
- Cargo fmt
- Cargo Audit
- Python pytest for worker code

Infrastructure:

- Git
- GitHub Actions
- Docker where justified
- WSL2 or Linux for supported isolation environments

Do not install all dependencies immediately.

Do not add frameworks solely because they are popular.

Each dependency must have a documented engineering purpose.

## 12. Research competitor analysis

Conduct a systematic, source-grounded comparison against relevant research and open-source projects.

Investigate:

- Google DeepMind AlphaEvolve
- Google DeepMind FunSearch
- OpenAI PaperBench
- OpenAI MLE-bench
- OpenAI Codex
- Anthropic long-running agent harnesses
- Anthropic agent evaluation research
- Sakana AI Scientist and AI Scientist v2
- Microsoft Discovery
- Turing software engineering agent research
- Stanford and other academic research agents
- FIRE-Bench
- Autonomous Science Bench
- Relevant automated machine learning systems
- Evolutionary computation frameworks
- Scientific workflow engines
- Reproducibility platforms
- Relevant Rust agent runtimes and ML frameworks

Search primary sources when web access is available:

- Official engineering blogs
- Research papers
- Official GitHub repositories
- Technical documentation
- Conference publications

For every relevant system, document:

1. Research objective
2. Architecture
3. Core algorithms
4. Implementation languages
5. Evaluation methodology
6. Strengths
7. Limitations
8. Computational requirements
9. Reproducibility
10. Overlap with AETHER
11. Potential improvements
12. Verified citations and publication dates

Do not fabricate citations, publication dates, repository capabilities or research results.

Separate confirmed findings, engineering interpretations and research hypotheses.

## 13. Target engineering organizations

AETHER is intended to demonstrate transferable technical competencies relevant to:

- OpenAI
- Anthropic
- Google DeepMind
- Google Research
- NVIDIA
- Microsoft Research
- Meta AI
- Amazon AWS
- Turing
- Hugging Face
- Sakana AI
- Databricks
- AMD
- Intel
- IBM Research
- Cloudflare

Create a career alignment document linking actual documented technical requirements to AETHER components and measurable portfolio deliverables.

Do not invent active vacancies or claim that this project guarantees employment.

Where public job descriptions are available, capture the source, date, required experience and actual eligibility constraints.

## 14. Create the repository structure

Create the following project structure, refining it where justified:

```text
AETHER/
├── README.md
├── AGENTS.md
├── PROJECT_CONTEXT.md
├── PROJECT_STATUS.md
├── CONTRIBUTING.md
├── SECURITY.md
├── CODE_OF_CONDUCT.md
├── CHANGELOG.md
├── CITATION.cff
├── LICENSE_DECISION.md
├── .gitignore
├── .editorconfig
│
├── docs/
│   ├── index.md
│   ├── archive/
│   │   └── v0.2/
│   ├── product/
│   ├── architecture/
│   ├── engineering/
│   ├── research/
│   ├── security/
│   ├── design/
│   ├── testing/
│   ├── operations/
│   ├── governance/
│   ├── decisions/
│   ├── learning/
│   ├── career/
│   └── diagrams/
│
├── research/
│   ├── literature/
│   ├── hypotheses/
│   ├── protocols/
│   ├── experiments/
│   ├── benchmarks/
│   ├── results/
│   └── publications/
│
├── contracts/
│   ├── schemas/
│   ├── examples/
│   └── README.md
│
├── crates/
│   ├── aether-core/
│   ├── aether-engine/
│   ├── aether-store/
│   ├── aether-runner/
│   ├── aether-cli/
│   └── aether-api/
│
├── workers/
│   └── python/
├── taskpacks/
├── benchmarks/
├── tests/
├── infra/
├── scripts/
├── web/
└── .github/
```

Implementation directories may contain explanatory README files during the documentation phase.

Do not fill them with speculative application code.

Do not declare unimplemented crates as active Cargo workspace members.

Preserve the root architecture and document future expansion points.

## 15. Mandatory documentation deliverables

Create substantive, technically precise documents, not placeholder headings or generic AI-written descriptions.

### Product documentation

- PRODUCT_REQUIREMENTS.md
- PROJECT_CHARTER.md
- PRODUCT_VISION.md
- USER_PERSONAS.md
- USE_CASES.md
- COMPUTER_APPLICATIONS.md
- FUNCTIONAL_REQUIREMENTS.md
- NON_FUNCTIONAL_REQUIREMENTS.md
- MVP_SCOPE.md
- SUCCESS_METRICS.md

### Architecture documentation

- HIGH_LEVEL_DESIGN.md
- LOW_LEVEL_DESIGN.md
- SYSTEM_ARCHITECTURE.md
- SOFTWARE_ARCHITECTURE.md
- COMPONENT_ARCHITECTURE.md
- DATA_ARCHITECTURE.md
- DATABASE_DESIGN.md
- API_DESIGN.md
- EVENT_ARCHITECTURE.md
- WORKER_PROTOCOL.md
- STATE_MACHINES.md
- RESOURCE_MANAGEMENT.md
- FAILURE_AND_RECOVERY.md
- SCALABILITY_DESIGN.md
- DEPLOYMENT_ARCHITECTURE.md
- HARDWARE_ARCHITECTURE.md

### Technical engineering documentation

- TECHNICAL_REQUIREMENTS.md
- SOFTWARE_REQUIREMENTS_SPECIFICATION.md
- TECHNOLOGY_STACK.md
- RUST_ENGINEERING_STANDARD.md
- CODING_STANDARDS.md
- ERROR_HANDLING.md
- CONCURRENCY_MODEL.md
- MEMORY_MANAGEMENT.md
- PERFORMANCE_ENGINEERING.md
- DEPENDENCY_POLICY.md
- REPOSITORY_STRUCTURE.md

### Research documentation

- RESEARCH_VISION.md
- LITERATURE_REVIEW.md
- RELATED_WORK.md
- COMPETITOR_ANALYSIS.md
- RESEARCH_GAPS.md
- ORIGINALITY_ASSESSMENT.md
- ECAR_SPECIFICATION.md
- MATHEMATICAL_FOUNDATIONS.md
- RESEARCH_HYPOTHESES.md
- EXPERIMENTAL_METHODOLOGY.md
- BENCHMARK_SPECIFICATION.md
- BASELINE_COMPARISON.md
- EVALUATION_METRICS.md
- ABLATION_STUDY_DESIGN.md
- STATISTICAL_ANALYSIS_PLAN.md
- REPRODUCIBILITY_PROTOCOL.md
- RESEARCH_ETHICS.md
- RESEARCH_PUBLICATION_ROADMAP.md
- REFERENCES.md

### Security documentation

- THREAT_MODEL.md
- SECURITY_ARCHITECTURE.md
- SANDBOX_DESIGN.md
- PERMISSION_MODEL.md
- DATA_PRIVACY.md
- SECRETS_MANAGEMENT.md
- SUPPLY_CHAIN_SECURITY.md
- INCIDENT_RESPONSE.md
- AI_AGENT_SAFETY.md
- PROMPT_INJECTION_DEFENSE.md

### Testing and quality documentation

- TEST_STRATEGY.md
- TEST_PLAN.md
- ACCEPTANCE_CRITERIA.md
- REQUIREMENTS_TRACEABILITY.md
- PERFORMANCE_TESTING.md
- FAILURE_INJECTION.md
- BENCHMARK_VALIDATION.md
- QUALITY_GATES.md

### Lifecycle and operations documentation

- SDLC.md
- RDLC.md
- DEVOPS_STRATEGY.md
- CI_CD_DESIGN.md
- OBSERVABILITY.md
- RELEASE_MANAGEMENT.md
- DISASTER_RECOVERY.md
- COST_MANAGEMENT.md
- DEVELOPMENT_ENVIRONMENT.md

Interpret RDLC as Research and Development Life Cycle.

### Governance documentation

- ARCHITECTURE_DECISION_RECORDS.md
- RISK_REGISTER.md
- DECISION_LOG.md
- DEVELOPMENT_POLICY.md
- LICENSE_EVALUATION.md
- RESEARCH_INTEGRITY_POLICY.md
- HUMAN_APPROVAL_POLICY.md

### Learning and career documentation

- BEGINNER_LEARNING_PATH.md
- RUST_ZERO_TO_ADVANCED.md
- COMPUTER_SCIENCE_FOUNDATIONS.md
- MATHEMATICS_FOR_AETHER.md
- AI_RESEARCH_CURRICULUM.md
- ENGINEERING_EXERCISES.md
- WEEKLY_LEARNING_PLAN.md
- TECHNICAL_ASSESSMENTS.md
- CAREER_ALIGNMENT.md
- PORTFOLIO_STRATEGY.md
- OPEN_SOURCE_STRATEGY.md
- INTERNSHIP_PREPARATION.md

### Roadmap documentation

- MASTER_ROADMAP.md
- MILESTONES.md
- IMPLEMENTATION_BACKLOG.md
- RELEASE_CRITERIA.md
- PHASE_ZERO.md

Consolidate overlapping content into canonical documents when appropriate. Use cross-references rather than maintaining contradictory copies.

## 16. Mandatory architecture diagrams

Create editable Mermaid or Graphviz source diagrams for:

- System context diagram
- Container architecture diagram
- Component architecture diagram
- Crate dependency graph
- ECAR research lifecycle
- Experiment execution sequence
- Worker communication sequence
- Evidence graph model
- Entity relationship diagram
- Campaign state machine
- Claim state machine
- Run lifecycle
- Budget reservation and settlement
- Security trust boundaries
- Sandbox architecture
- Distributed worker architecture
- Deployment architecture
- Failure recovery workflow
- Research verification pipeline
- User research journey
- CI/CD pipeline

Use standard notation and include explanatory captions.

Generate rendered SVG or PNG diagrams only if the necessary tooling is available.

Ensure diagrams agree with the written specifications.

## 17. Research evaluation framework

Preserve the proposed research configurations:

A. Single LLM proposal with trusted evaluation.

B. Iterative agent with fixed verification allowance.

C. Adaptive scheduling with a flat claim checklist.

D. ECAR with dependency-aware evidence graph and adaptive verification.

E. Optional graph-aware fixed scheduling to complete an ablation comparison.

All systems must use:

- Equivalent task sets
- Identical model versions where applicable
- Comparable tool access
- Identical trusted evaluation rules
- Comparable resource budgets
- Controlled datasets
- Recorded environment information
- Independent seeds
- Frozen evaluation protocols

Measure:

- Verified scientific utility
- Reproducibility
- False verification rate
- Claim coverage
- Experiment completion
- Token consumption
- Wall-clock time
- CPU utilization
- GPU utilization
- Memory consumption
- Cost
- Recovery behavior

Distinguish pilot experiments from held-out evaluation.

Determine sample sizes and statistical procedures after feasibility and variance estimation.

Do not invent performance results.

Do not fabricate research conclusions.

A negative or null result is a legitimate scientific outcome.

## 18. Security principles

AETHER will eventually execute research code generated or proposed by AI. This makes security a fundamental architectural requirement.

Treat model outputs, retrieved documents and worker outputs as untrusted data.

Implement the design around:

- Default-deny permissions
- Approved task registries
- Least privilege
- Network isolation
- Read-only input mounts
- Bounded output directories
- Resource limits
- Execution timeouts
- Secret isolation
- Human approval
- Artifact integrity
- Dependency verification
- Structured event logging
- Crash recovery
- Cancellation
- Budget limits

Never permit unrestricted generated shell commands.

Never allow model-generated instructions to override trusted policy.

Do not run experiments against arbitrary real-world systems.

Do not store credentials in project files.

Document limitations of Docker, WSL and local process isolation.

Use appropriately hardened isolation before permitting untrusted execution.

## 19. Development lifecycle

Organize the project into controlled phases.

### Phase 0 — Genesis

Documentation, environment assessment, research comparison, architecture approval and learning plan.

### Phase 1 — Foundation

Learn Rust fundamentals and implement deterministic experiment evaluation.

### Phase 2 — Scientific evidence

Implement typed manifests, content hashing, evidence relationships and reproducible experiment records.

### Phase 3 — Execution infrastructure

Implement reviewed workers, typed execution results, resource accounting and recovery.

### Phase 4 — ECAR

Implement and evaluate adaptive research scheduling.

### Phase 5 — Intelligent research

Integrate model adapters, hypothesis generation and literature retrieval.

### Phase 6 — Research validation

Conduct pilot experiments, controlled comparisons, ablation studies and held-out evaluations.

### Phase 7 — Advanced infrastructure

GPU profiling, native inference, concurrency and optional distributed execution.

### Phase 8 — Research interface

Develop a professional scientific workspace and developer integration interfaces.

### Phase 9 — Publication and release

Research report, reproducibility package, open-source release and portfolio development.

Create measurable entry criteria, exit criteria, deliverables, dependencies and approval gates for every phase.

Do not promise fixed completion dates without considering the owner's learning progress and available resources.

## 20. Persistent coding-agent context

Create a root AGENTS.md that future Codex sessions will automatically use.

It must contain:

- Project mission
- Approved architectural decisions
- Rust-first rule
- Documentation hierarchy
- Research integrity requirements
- Security constraints
- Repository conventions
- Testing expectations
- Human approval boundaries
- Learning and mentorship requirements
- Current implementation phase
- Instructions for resolving conflicting specifications

Create PROJECT_CONTEXT.md as a comprehensive project knowledge base.

Create PROJECT_STATUS.md containing:

- Current phase
- Completed work
- Pending work
- Approved decisions
- Open questions
- Known blockers
- Next recommended task
- Tests actually executed
- Research experiments actually completed

Create docs/decisions/ with individually numbered architecture decision records.

Future agents must read these documents before modifying the system.

Maintain the context documents as development proceeds.

Never silently change architectural decisions.

## 21. Engineering quality rules

Follow these principles:

- No fake implementation.
- No fake research results.
- No invented citations.
- No unnecessary abstractions.
- No unexplained dependencies.
- No premature microservices.
- No speculative distributed infrastructure in the initial release.
- No duplicated sources of truth.
- No insecure execution defaults.
- No meaningless placeholder documentation.
- No fabricated benchmark graphs.
- No hidden changes to project scope.
- No architecture decisions based solely on hype.

Prefer correctness, clarity, testability, scientific rigor and maintainability over visual complexity.

Each functional requirement must have a unique ID and measurable acceptance criteria.

Each major architectural decision must describe alternatives and trade-offs.

Each research hypothesis must have a validation methodology.

Each milestone must have a definition of done.

Use professional technical language suitable for researchers and senior engineers, with separate beginner-friendly explanations in the learning documentation.

## 22. First native implementation milestone specification

Define, but do not yet implement, the first native milestone:

A minimal Rust CLI that evaluates a deterministic classification experiment.

It should eventually demonstrate:

- Functions
- Variables
- Vectors
- Borrowing
- Ownership
- Structs
- Enums
- Result-based error handling
- Unit testing
- Deterministic metric calculation
- Input validation

Expected initial commands:

```text
aether init
aether run-fixture
aether verify <run-id>
aether report <campaign-id>
```

The first milestone must not require Python, an external LLM, a GPU or a paid API.

Only activate commands as their implementation milestones are completed.

Document expected inputs, outputs, failure cases and acceptance tests.

## 23. Validation before completion

After creating the workspace:

1. Inspect all generated documents.
2. Compare them against the source design authority.
3. Detect conflicting requirements.
4. Detect architecture inconsistencies.
5. Validate Markdown relative links.
6. Validate JSON syntax and available schemas.
7. Validate Mermaid diagrams where tooling permits.
8. Check that every major requirement has a verification method.
9. Review the threat model.
10. Review research originality claims.
11. Confirm that proposed results are clearly labeled as hypothetical.
12. Inspect the repository for secrets.
13. Check that source documentation remains unchanged.
14. Verify the final directory structure.
15. Create a documentation audit report.

Initialize a local Git repository if one does not exist.

Do not create a remote repository, publish files, push commits or change system settings without explicit permission.

## 24. Final deliverables

At the end of this task, provide:

- Exact project directory path
- Complete folder tree
- Documentation inventory
- Imported source document inventory
- Architecture summary
- Research contribution summary
- Proposed originality assessment
- Hardware compatibility assessment
- Important technical decisions
- Research and engineering risk summary
- Documentation validation results
- Missing information and blockers
- Unresolved design questions
- Next recommended learning lesson
- First milestone acceptance criteria

Create a consolidated file:

`docs/AETHER_MASTER_BLUEPRINT.md`

It should act as the primary navigation document for the complete project.

If PDF-generation tooling is available, optionally produce a professionally formatted PDF edition.

Do not claim PDF generation or validation succeeded unless it actually did.

## 25. Execution order

Begin with:

1. Inspect the environment.
2. Locate the existing AETHER v0.2 documents.
3. Create the project directory.
4. Preserve and read the original design authority.
5. Perform the research and architecture gap analysis.
6. Create the complete engineering documentation.
7. Create persistent agent context.
8. Initialize the documentation-first repository.
9. Perform consistency and quality validation.
10. Present the final design review.

Do not proceed into application implementation.

Ask the owner questions at meaningful design decision points, not for information that has already been supplied.

Use justified defaults for reversible decisions and document them.

The final objective of this task is to establish a rigorous, coherent and navigable engineering foundation from which AETHER can be built and learned systematically.

BEGIN PROJECT GENESIS.