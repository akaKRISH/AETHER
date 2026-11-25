# Reproducibility protocol

## Required evidence bundle

- Frozen protocol and approval digest, policy version, task-pack version, dataset and split digests, baseline version, declared seeds and actual run seed.
- Source revision and content digest, toolchain, dependency lockfiles, platform and CPU/GPU details, container image digest if used, thread settings and evaluator version/digest.
- Run and attempt identifiers, parent/replay linkage, bounded logs and digests, metrics and their units, artifact roles and SHA-256 manifests, budgets and resource readings with accounting units, terminal state and reason.
- Claim predicate, required evidence by ID, verifier version, decision reason and any later invalidation events.
- When a model is used, declared model/version, sampling configuration and safe prompt digest. Release prompt content only after license, privacy and policy review.

The reproducer starts in a clean directory with the pinned inputs and fresh attempt identity, confirms hashes and environment compatibility, reruns the approved task, and obtains metrics from the trusted evaluator. Compare on the registered criterion. Exact integer counts are appropriate for the deterministic classification fixture. Use frozen tolerances for floating point or controlled timing. A reused output artifact can verify integrity and value, while a fresh execution with the stated independence conditions tests repeatability. Require a more independent implementation or external team when testing scientific generalizability, and label evidence scope.

Keep untrusted worker outputs outside the evidence store until checks pass. If a hash, required artifact, provenance item or environment precondition fails, record the exact reason and withhold verification. A valid discrepancy gets a disconfirming record. Append superseding decisions and preserve historical report visibility.

Package only releasable data. Include environment setup instructions, baseline, protocol, analysis version, all attempt summaries and a manifest. Publish immutable result identity after approval. For a later export mapping, model executions and outputs using the [Workflow Run RO-Crate profiles](https://www.researchobject.org/workflow-run-crate/).
