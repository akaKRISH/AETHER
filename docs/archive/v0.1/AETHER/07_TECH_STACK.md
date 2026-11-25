# 07 | Technology stack and tradeoffs

| Layer | R0/R1 choice | Later option / trigger |
|---|---|---|
| Language | Python 3.12; typed modules | C++ only for measured hot path. |
| Packaging | uv or venv + pip; lock dependencies | Container image digest for replay. |
| Validation | Pydantic v2, JSON Schema | Maintain versioned contracts. |
| CLI | argparse; rich output optional | Typer only if CLI complexity grows. |
| Persistence | SQLite WAL, SQLAlchemy 2 / migrations | PostgreSQL for concurrent service. |
| Artifacts | Local SHA-256 blob store | S3-compatible store with access control. |
| ML | NumPy, scikit-learn, PyTorch optional | ONNX / llama.cpp / CUDA only when needed. |
| Model interface | Custom adapter, scripted/mock first | Local quantized model or approved provider. |
| Retrieval | Curated local sources, citation metadata | OpenAlex / arXiv adapter; pgvector if justified. |
| Runtime | Linux isolated rootless containers; trusted tasks first | Dedicated VM or microVM for untrusted code. |
| API | FastAPI in R2 | SSE for progress, not socket complexity. |
| UI | React + TypeScript + Vite in R2 | Graph visualization via Cytoscape if needed. |
| Observability | JSON logs, run metrics, timestamps | OpenTelemetry / Prometheus after remote workers. |
| QA | pytest, Ruff, mypy, GitHub Actions | Property tests, Playwright for UI. |

## Version and cost policy
These are selected technologies, not claims about current latest versions. Test Python version against ML packages, CUDA/driver compatibility and the available container runtime. Freeze exact dependency versions with hashes, pin image digests and record OS/driver versions in manifests. Do not install large stacks before the first CLI task works. Prefer no paid service; verify cloud credit availability and terms before use. If model usage cannot be bounded or measured, use mock/local experiments or exclude those runs from cost-matched research comparisons.

## Alternatives rejected for R0
LangGraph: useful later, but core scheduling is the research object and should be understandable from scratch. Kubernetes/Ray: unnecessary single-worker complexity. Neo4j: relational edges and indexes suffice. Vector DB: no retrieval bottleneck yet. Training a large foundation model: exceeds hardware and is irrelevant to the first hypothesis. Docker alone: not a guarantee against hostile code. All alternatives can be revisited through ADRs and profiling.
