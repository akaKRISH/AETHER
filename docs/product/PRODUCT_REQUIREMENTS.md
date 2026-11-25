# Product requirements

AETHER turns a bounded objective into a reproducible computational record. The user selects a task pack, previews the frozen evaluator, declared resources and policy, authorizes the plan, then inspects results and their evidence. The local research release includes cancellation and restart recovery. External-facing interfaces are later milestones.

The [functional requirements](FUNCTIONAL_REQUIREMENTS.md) and [non-functional requirements](NON_FUNCTIONAL_REQUIREMENTS.md) own release promises. [MVP scope](MVP_SCOPE.md) defines the first slice and the wider local release. [Measures](SUCCESS_METRICS.md) define evidence for judging success.

Inputs include an approved task ID, protocol revision, resource caps, evaluator version and data digests. Outputs include immutable attempt IDs, redacted logs, a typed result and evidence links. A rejected or inconclusive claim remains visible in reports. The starter fixture illustrates the flow using synthetic labels and predictions.
