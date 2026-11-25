# CI/CD design

| Stage | Proposed gate |
|---|---|
| Genesis | Validate links, schemas, diagrams, traceability and archive bytes locally |
| Native crates | Pinned toolchain; `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo test --workspace --locked`; test no-Python fixture separately |
| Reviewed workers | Isolated Linux job with deny-by-default permissions, denial tests, timeout and bounded fixtures |
| Dependency review | Record RustSec audit, license inventory, exact package and action pins, and scan tracked inputs for secrets |
| Release | Owner reviews tests, docs, compatibility, provenance and permissions. Apply signing/publishing only under explicitly approved scope |

GitHub Actions workflows are future work. In particular, untrusted proposed patches receive no privileged secrets. [Release management](RELEASE_MANAGEMENT.md)
