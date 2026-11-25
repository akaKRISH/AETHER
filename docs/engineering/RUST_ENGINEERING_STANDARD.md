# Rust engineering standard

Use Rust Edition 2024 and stable tooling during implementation. Begin with `aether-core` and `aether-cli`. Keep core deterministic and free of filesystem, clock, database, shell, HTTP and async-runtime APIs. Inject values such as timestamps and configuration at the boundary where applicable.

Represent state with enums and exhaustive matching, identities with validated newtypes where accidental mixing is material, and read-only inputs with borrowed slices. Use private fields for invariants and typed constructors for fallible parsing. Domain methods return `Result` for recoverable errors. Document checked conversions and preserve integer budget accounting.

Prefer one clear Rust expression where it communicates the full operation. Reuse the standard library and project types. Propagate straightforward errors with `?`, borrow shared data, and use a direct iterator or pattern match when it improves readability. Validate equal lengths and nonempty inputs before counting matches with `zip`. Keep transaction boundaries, checked arithmetic, error context and state transitions visible. Let rustfmt determine physical line breaks. Assess optimizations against a measured workload and keep unrelated source edits out of the change.

Enforce `#![forbid(unsafe_code)]` in first-party core. Reconsider FFI only under a reviewed boundary with clear ownership and soundness evidence. No Python interpreter is required to build or run the fixture. Format with rustfmt, lint with Clippy, test invariant cases and inspect Cargo metadata. Add release locking only when a valid Cargo workspace exists.

Mentorship checkpoint. Explain who owns a prediction `Vec`, why its accuracy caller borrows `&[u8]` and how an `Err` prevents evidence creation when lengths differ. Source [native implementation contract](../archive/v0.2/AETHER/21_RUST_ENGINEERING.md).
