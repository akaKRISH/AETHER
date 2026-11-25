# Data architecture

A project defines immutable protocol revisions. A campaign pins protocol, budget and policy. A run identifies a logical research action. An attempt identifies one execution and its observed costs. A metric is a trusted observation with units, evaluator identity and data split. A claim applies a frozen predicate to trusted observations.

| Data class | Rule |
|---|---|
| Scientific definition | Freeze task, rules, seeds and tolerances before evaluated execution |
| Runtime state | Update lease, usage and outcomes with fences and idempotence |
| Evidence | Bound intake and record immutable attribution |
| Derived view | Query committed evidence and explain limitations |
| External suggestion | Treat as data under policy |

Use typed `SUPPORTS`, `CONTRADICTS`, `DEPENDS_ON` and `REPRODUCES` associations. Derive provenance such as `GENERATED_BY` from explicit references initially. `A DEPENDS_ON B` means A requires B's eligible current verdict. Reject insertion if B already reaches A, and reproject dependent claims after upstream revocation.

Hash exact artifact bytes with SHA-256. Manifests record task/dataset/split/code digests, locks or image, seeds, policy, evaluator, hardware, failures and resource observations. Define versioned canonical encoding before cross-language protocol hashing. Evaluate [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785.html) with duplicate-key rejection and exact integer constraints.

See [database design](DATABASE_DESIGN.md), [evidence graph](../diagrams/evidence-graph.mmd) and [source data design](../archive/v0.2/AETHER/08_DATA_DESIGN.md).
