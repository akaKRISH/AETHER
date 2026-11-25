# Rust from zero to advanced

Write and run one exercise at a time in the [practice workspace](../../learning/README.md). Review the code, reasoning and output before advancing.

| Lesson | Learn | Apply to Milestone 01 |
|---|---|---|
| 1 | Program structure, `main`, `println!` and Cargo | Display the fixture identity and sample count |
| 2 | Variables, names and immutability | Represent correct and total sample counts as named values |
| 3 | Integers, floating-point values, booleans and text | Choose types for counts, labels, matches and display values |
| 4 | Explicit numeric conversions and their limits | Convert small counts to a floating-point representation when needed for display |
| 5 | Arithmetic, integer and floating-point division | Derive accuracy while retaining exact counts for replay |
| 6 | Function parameters, return types and expressions | Separate the calculation from terminal output |
| 7 | Conditions and input validation | Detect a zero total or an impossible count before division |

Once these foundations have been reviewed, learn arrays, vectors and slices. Practice ownership and borrowing when passing predictions and labels to the evaluator. Introduce structs, enums, `Option` and `Result` as the exercise needs to represent outcomes and errors. Use tests to check known counts, empty inputs, unequal lengths and invalid labels before approving a production evaluator slice. Hashes, manifests and replay follow a reviewed deterministic calculation. [Milestone 01](../roadmap/MILESTONE_01_NATIVE_FIXTURE.md)

Later milestones cover transactions, reviewed worker adapters and concurrency when the design needs them. [Rust Book](https://doc.rust-lang.org/book/) · [Cargo Book](https://doc.rust-lang.org/cargo/)
