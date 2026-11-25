# Dependency policy

Evaluate dependencies when a milestone has an observable need. Check existing code and standard-library support first. Document owner, features, transitive footprint, platform and toolchain support, maintenance, licensing, security advisories, source provenance and exit cost. Compare a simpler option and attach a test that justifies adoption.

| Candidate | First evaluation gate |
|---|---|
| Clap, Serde, serde_json, sha2 | Verify supported Windows/Linux targets and fmt/lint/test lock with minimal features |
| rusqlite | Decide system versus bundled SQLite, verify linked source against advisories and run migration/restore |
| Optional Python scientific stack | Pin Python and worker dependencies and validate isolated CPU wheels and evaluator agreement |
| Candle, Burn or ONNX Runtime | Profile accuracy/latency/VRAM and review native dependency and model terms |
| Tokio, Axum, SQLx | Demonstrate concurrency or hosting need and test service failure handling |

Commit Cargo.lock when application packages exist. Pin toolchain and CI action identifiers at implementation review, require review of dependency upgrades and audit release targets. Pin separately locked Python dependencies and image digests for reviewed workers. Database and runtime dependencies are verified independently of current versions on the development host.

See [technology choices](TECHNOLOGY_STACK.md), [RustSec](https://rustsec.org/) and [source policy](../archive/v0.2/AETHER/07_TECH_STACK.md).
