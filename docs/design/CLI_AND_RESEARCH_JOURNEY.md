# CLI and research journey

Phase 1 serves one local learner. Commands represent proposed entry points, activated in reviewed slices.

| Command | User expectation | Representative failure |
|---|---|---|
| `aether init` | Create a workspace at current directory without modifying unrelated files | Existing conflicting workspace marker |
| `aether run-fixture` | Run the built-in labels and predictions, return artifact identity and accuracy 0.75 | Uninitialized directory or I/O failure |
| `aether verify <run-id>` | Check manifest/input digests and reproduce 3/4 | Unknown ID, changed bytes or unsupported manifest version |
| `aether report <campaign-id>` | List validated fixture results with provenance and limits | Missing campaign or invalid referenced manifest |

An exit code has a stable meaning across commands. Show machine-consumable output in an agreed JSON mode and route human diagnostics separately. Output derives from recorded counts. Start with the first approved command slice and update help when each command becomes available.

The [v0.2 UX](../archive/v0.2/AETHER/13_UX_APPLICATIONS.md) sketches protocol validation, approval, campaign status, cancellation, claim verification and export. The [native contract](../archive/v0.2/AETHER/21_RUST_ENGINEERING.md) and Genesis govern the first command set, including run-ID replay. Later claim verification needs a distinct unambiguous command interface and explicit approval.

R2 can present mission setup, campaign monitor, experiment explorer, evidence graph and report review. Show missing evidence, stalled connection and failure explicitly. Use labels alongside status colors, keyboard access and visible focus. Evaluation labels stay protected. See [CLI error handling](../engineering/ERROR_HANDLING.md) and the [user journey](../diagrams/user-journey.mmd).
