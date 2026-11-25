# Technology stack

Rust Edition 2024 is the design authority for trusted code. Edition 2024 was released with Rust 1.85.0. Pin a tested toolchain and dependency lock when implementation begins. In a virtual workspace set resolver = "3" explicitly. [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) · [Cargo resolver](https://doc.rust-lang.org/cargo/reference/resolver.html)

| Need | Proposed technology | Adoption trigger |
|---|---|---|
| Fixture | Rust core, Cargo, standard library | Phase 1 owner review |
| Command flags and JSON output | Clap, Serde, serde_json | A real CLI parse/serialization need |
| Errors and digest | Typed errors, thiserror if it reduces repetition, sha2 | First typed boundaries and manifests |
| Transactions | SQLite and rusqlite | Phase 2 with tested migration, backup and supported SQLite source |
| Reviewable worker | std::process, approved Linux isolation | Phase 3 enforcement is verified |
| Scientific libraries | Reviewed Python NumPy, SciPy, scikit-learn or PyTorch worker | Necessary scientific task, versioned files and separate locked environment |
| Native ML | Candle, Burn or ONNX Runtime | Measured numerical parity, dependency licensing and memory on target host |
| Async/API/UI | Tokio, Axum, later React/TypeScript | Concurrent I/O or validated workflow demand |
| Hosted storage/services | PostgreSQL, SQLx, remote artifacts | Local bottleneck and explicit infrastructure approval |

FFI and embedded Python are deferred. The initial system has zero GPU, model provider and cloud obligations. No dependency is adopted during Genesis. Use the [dependency policy](DEPENDENCY_POLICY.md) before pinning versions. [Source stack](../archive/v0.2/AETHER/07_TECH_STACK.md)
