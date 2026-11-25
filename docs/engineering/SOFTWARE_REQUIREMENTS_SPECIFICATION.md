# Software requirements specification

AETHER accepts an owner-approved computational research task, executes reviewed evidence-producing actions within bounds, evaluates outputs and reports reproducible findings. Operational correctness requires approval, resource and integrity invariants across normal execution, errors and recovery.

| Requirement area | Observable behavior |
|---|---|
| Inputs | Reject malformed input, invalid IDs, unknown task/policy and conflicting hash before allocation |
| Configuration | Persist immutable approved protocol and policy per campaign |
| Attempt | Assign unique identity and pin seed, dataset, task version, evaluator and code digest before dispatch |
| Results | Validate shape, resource use, provenance and fence before publication |
| Verification | Protected evaluator determines metric and frozen-rule outcome, only eligible claims enter VERIFIED |
| Recovery | Persist typed failure, terminalize and settle attempt once, retry only eligible infrastructure failure |
| Report | Read committed state, show evidence, negative outcomes and unresolved gaps |

Functional IDs live in [functional requirements](../product/FUNCTIONAL_REQUIREMENTS.md) and TRD-N IDs in [technical requirements](TECHNICAL_REQUIREMENTS.md). Apply the [security threat model](../security/THREAT_MODEL.md) before admitting work. Phase 1 only admits the built-in native fixture. Test exact 3/4 matches, unequal length, empty and invalid labels, replay integrity, safe local initialization and bounded reporting. Use the [low-level design](../architecture/LOW_LEVEL_DESIGN.md) for inputs and outputs.
