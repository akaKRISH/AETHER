# AETHER Rust practice workspace

This standalone Cargo workspace holds student exercises for [Milestone 01](../docs/roadmap/MILESTONE_01_NATIVE_FIXTURE.md). Each file under `src/bin/` is its own executable. The workspace uses Rust Edition 2024 and the standard library. Production crates live under `crates/` at the repository root.

## Current exercise

Read [Lesson 1](../docs/learning/LESSON_01_CLASSIFICATION_AND_RUST.md#first-exercise-and-quiz), then replace the [starter](src/bin/lesson_01.rs) with your own program. From the AETHER repository root in PowerShell, run:

```powershell
cargo run --manifest-path .\learning\Cargo.toml --bin lesson_01
```

Predict the output before running. Share your code and terminal output for review. If compilation fails, share the full error and the code that produced it. The [lesson sequence](../docs/learning/RUST_ZERO_TO_ADVANCED.md) shows how we will build toward the evaluator.
