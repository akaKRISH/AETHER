# Threat model

Scope runs from the local Rust fixture through the approved-worker design. Release authorization is separate for each deployment profile. The protected assets are campaign and budget integrity, a hidden evaluator and labels, credentials, host files, licensed inputs, and the reproducibility record. Trust the human approver, controller and policy, trusted evaluator, narrowly scoped ingestion, and reviewed task packs. Treat proposals, retrieved content, user-supplied manifests, worker outputs and artifacts as untrusted data. A task pack's review approves its pinned version and capabilities. Physical compromise and an actively malicious host administrator are outside the initial isolation guarantee.

| ID | Threat and control | Required evidence |
| --- | --- | --- |
| SEC-01 | An injected instruction tries to obtain authority. Treat text as data and validate structured requests against controller policy. | Inert fixture requests a denied capability in retrieved text. Controller emits denial and performs zero side effects. |
| SEC-02 | A worker reads or affects the host. Run as an unprivileged identity with an isolated workspace and explicitly mapped files. | Test-only canaries in host files, environment and control interfaces are inaccessible. |
| SEC-03 | A worker transmits data. Disable worker network at the isolation boundary. | Controlled local DNS and test endpoint receive zero traffic. |
| SEC-04 | A task exhausts resources. Bound processes, memory, disk, time and output. Supervise the whole process tree. | Benign over-limit fixtures terminate and account for usage. No descendant survives. |
| SEC-05 | Candidate output poisons scientific evidence. Keep labels and grading isolated. Verify artifact hashes and recompute metrics independently. | Valid but incorrect predictions fail trusted grading. Malformed predictions are rejected. A fabricated verdict and changed artifact cannot reach VERIFIED. |
| SEC-06 | Unsafe output path or archive threatens the host. Resolve only safe declared artifacts within the private output root. | Traversal, symlink, reparse-point and oversized output cases fail with no external read or write. |
| SEC-07 | Stale approval or forged result starts work or commits state. Check approval scope, hash, caps and lease token. | Modified config denies execution and late completion changes no ledger state. |
| SEC-08 | A dependency or executable changes without review. Pin and verify build and runtime inputs. | Unknown executable and digest drift fail. Release inventory has license and vulnerability dispositions. |
| SEC-09 | Sensitive information leaks into persisted or exported evidence. Minimize and segregate data and redact exports. | Synthetic secret and sensitive fields are absent in shareable bundles. |
| SEC-10 | Autonomy outlives owner consent. Enforce finite budgets, cancellation, domain restrictions and publish gates. | Cancel or budget exhaustion stops further dispatch. Neither worker nor proposer can publish or modify policy. |

The Windows native fixture uses trusted code and fixed synthetic inputs. Worker isolation remains unproven until scoped denial tests pass on the selected runtime. A crash or missing isolation capability fails closed and triggers the [incident process](INCIDENT_RESPONSE.md) when compromise could have occurred.

Source authority is [v0.2 Security](../archive/v0.2/AETHER/12_SECURITY.md). See [isolation](SANDBOX_DESIGN.md) and [permissions](PERMISSION_MODEL.md).
