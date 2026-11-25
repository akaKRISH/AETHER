# Permission model

Every operation must have a recognized action, a matching capability and current authorization. A model's proposed action, a worker result and documentation can request consideration only. Type and schema checks precede policy and domain checks. Re-evaluate authorization at dispatch and check lease validity at result commit.

| Principal | Allowed authority | Boundary |
| --- | --- | --- |
| Owner | Approve scoped campaign, budget, task pack, exceptional scope transition and release | Approval requires a recorded decision. Local owner is the only initial approver. |
| Controller | Validate policy, reserve and settle budget, append evidence events | Uses approved bounds and rejects unknown versions |
| Runner | Execute task registry selection in the matching runtime profile, emit observations | No budget, ledger or evaluator authority |
| Evaluator | Compute metrics and verdict from frozen protocol and validated inputs | Writes only via trusted controller pathway |
| Proposer | Return typed candidate suggestions | No direct file, network, budget or process authority |
| Export path | Read allowlisted evidence and prepare local report | Transfer and publication require independent explicit owner authorization |

Proposed approval binding includes the immutable protocol and policy digests, task version, output/data scope, budget cap vector and applicable expiry or invalidation. Credential handles never appear in approvals. Changes to authorized fields require a new decision. Stale worker output is rejected even when its envelope is otherwise valid. When parsing, policy lookup, or environment qualification fails, leave protected state unchanged and log a typed denial. [Human approval policy](../governance/HUMAN_APPROVAL_POLICY.md)
