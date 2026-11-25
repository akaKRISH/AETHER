# Lesson 1 — how a Rust program starts

Rust programs are grouped into packages. A package's `Cargo.toml` tells Cargo its name, edition and dependencies. Cargo looks for the default executable in `src/main.rs`. It also builds a separate executable for each file under `src/bin/`. When an executable starts, Rust calls its `main`.

```rust
fn main() {
    println!("AETHER native preflight");
}
```

`fn` declares a function. The braces enclose its body. `println!` is a Rust macro that writes a line to the terminal. The semicolon ends the statement. With two print statements in the body, the first runs before the second.

AETHER's `aether-cli` will contain `main` and handle what the user types and sees. `aether-core` will be a library that the CLI can call to evaluate predictions. Keeping scoring in a library lets us test it without running a command or reading files. [Milestone design](../roadmap/MILESTONE_01_NATIVE_FIXTURE.md) · [Rust Book, Hello World](https://doc.rust-lang.org/book/ch01-02-hello-world.html)

## First exercise and quiz

Replace the [Lesson 1 starter](../../learning/src/bin/lesson_01.rs) with your own complete program. Use two `println!` statements to print these lines in order.

```text
Experiment: classification-v1
Samples: 4
```

Follow the [workspace run instructions](../../learning/README.md). Share your code and terminal output, or the compiler error if it fails. Name the function Rust calls when the executable starts and predict the effect of swapping the two print statements.

**Learning progress:** Workspace ready. Student attempt, local run and review pending.
