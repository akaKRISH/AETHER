# Risk register

Ratings estimate likelihood and impact before mitigation. They are design judgments recorded at Genesis. The project owner is accountable. Future reviewer roles in this register require named assignment before their review gates can pass.

| ID | Risk | L / I | Mitigation, trigger and evidence | Responsible role |
| --- | --- | --- | --- | --- |
| RSK-01 | Originality overlaps existing work | High / High | Complete literature update before registration. Narrow the contribution to a comparison or replication if direct prior work subsumes the claim. | Research lead |
| RSK-02 | Scope and deadlines exceed developer capacity | High / High | Deliver tested local slices with lessons from the implemented code. Maintain a short ready backlog and keep optional integrations gated. | Owner |
| RSK-03 | Target hardware or free credits insufficient | Medium / High | Measure baseline on CPU and keep a local scripted path. Qualify memory and storage before expanding the pilot. | Engineer |
| RSK-04 | Provider cost or model drift undermines comparison | High / Medium | Use the same pinned provider/model within comparisons, preserve usage reports and stop on unresolved overrun. Use a deterministic control when feasible. | Research lead |
| RSK-05 | False verification or held-out leakage | Medium / Critical | Independent evaluator, protected labels, replay, contamination checks. Halt study and append protocol correction on confirmed leakage. | Evaluator reviewer |
| RSK-06 | Isolation or process cleanup fails | Medium / Critical | Restrict to trusted fixture until denial tests pass on the target worker profile. Suspend affected runtime after any cleanup uncertainty. | Security reviewer |
| RSK-07 | Study is underpowered | High / Medium | Pilot paired differences and document precision or power review before freeze. Report effect and uncertainty even when indeterminate. | Research lead |
| RSK-08 | Redistribution rights are missing | Medium / High | Require asset-by-asset inventory and owner review. Exclude unresolved material from distribution and document its replacement path. | Owner |
| RSK-09 | Native Rust learning or ML interoperability delays progress | Medium / Medium | Begin with pure synchronous core. Add optional reviewed workers only after contract and numerical parity tests. | Engineer |
| RSK-10 | Local event/artifact consistency degrades across crashes | Medium / High | Pair database changes transactionally, reconcile leases and verify restored artifacts by digest. The local SQLite and protocol restore reproduced the recorded campaign export. [Validation](../testing/TEST_RESULTS.md) | Engineer |
| RSK-11 | Release assurances outrun evidence | Medium / High | Distinguish planned tests from observed outcomes. Require independently reviewed reproduction before labelling a bundle independently reproducible. | Release reviewer |

Review risks at milestone entry, after incidents and before study/publication approval. Record any accepted residual risk in the relevant decision with scope, sign-off and the next review trigger. [v0.2 governance](../archive/v0.2/AETHER/18_GOVERNANCE_RISKS.md)
