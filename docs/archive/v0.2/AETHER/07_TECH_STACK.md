# 07 | Rust-first technology stack and tradeoffs

| Layer | R0/R1 choice | Later option / trigger |
|---|---|---|
| Language | Stable Rust, edition 2024, Cargo workspace | Python for isolated ML workloads only. |
| Toolchain | rustup, rustfmt, Clippy, Cargo.lock | Pinned tested toolchain in rust-toolchain.toml. |
| CLI | clap; structured JSON output | ratatui only if TUI needed. |
| Contracts | serde, serde_json; JSON Schema | Versioned worker protocol. |
| Errors / logs | thiserror, tracing | anyhow at CLI boundary; OpenTelemetry later. |
| Persistence | SQLite WAL via rusqlite; migrations | PostgreSQL with sqlx if concurrent server demands it. |
| Artifacts | SHA-256 via sha2; local blob store | S3-compatible storage. |
| Runtime | std process for reviewed tasks; Linux isolation supervisor | Tokio for concurrent I/O, dedicated VM for untrusted code. |
| ML | Isolated Python, scikit-learn, PyTorch, NumPy | Evaluate Candle, Burn or ONNX Runtime for native inference; CUDA only after profiling. |
| Model adapters | Scripted mock; reqwest for approved HTTP providers | Local llama.cpp service; optional Rust inference. |
| API | Axum + Tokio in R2 | SSE for progress. |
| UI | React + TypeScript + Vite in R2 | Tauri desktop only if warranted. |
| Testing | cargo test, proptest, cargo audit, GitHub Actions | cargo-nextest, cargo-llvm-cov, Criterion; pytest for Python workers. |

## Policy
Pin a tested Rust toolchain and Cargo.lock for applications, worker dependency locks and image digests. Verify versions, licenses, platform and GPU compatibility before use. Keep dependencies minimal. No paid services without explicit approval; cloud credits are conditional. Rust does not guarantee speed, correctness or isolation; measure before optimizing. Python ML libraries remain appropriate because native ML coverage and numerical parity require testing. Avoid PyO3 in R0: a subprocess is simpler and keeps failure domains separate. PyO3/maturin may expose trusted Rust libraries to Python researchers later, not run untrusted workloads in the controller.

## Rejected initial complexity
No mandatory agent framework, Kubernetes, distributed queue, graph database, native model trainer, or custom GPU kernels. Native Rust inference is an optional extension with measured parity, not a prerequisite. The scientific research method remains language-independent.
