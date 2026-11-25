# 12 | Security, safety and responsible research

## Trust model
Trusted: human approver, controller, policy, evaluator, artifact ingestion, task pack after review. Untrusted: model outputs, generated code, external papers, candidate datasets and sandbox outputs. Candidate never receives provider keys, hidden labels, signing keys, host mounts or controller database. R0 runs only reviewed templates. Do not run arbitrary untrusted code on personal hardware; use dedicated VM or hardened isolated environment after review. Containers are defense-in-depth rather than a sufficient standalone isolation guarantee.

| ID | Threat | Control / test |
|---|---|---|
| SEC-01 | Prompt injection from paper or tool | Treat retrieval as data; allowlisted actions; test malicious instruction fixture. |
| SEC-02 | Host escape / secret theft | No socket, privileged mode, credentials, writable host mount; test denial. |
| SEC-03 | Network exfiltration | Network disabled in runner; explicit controller adapters only. Test egress. |
| SEC-04 | Resource exhaustion | CPU/memory/PID/disk/time quotas; kill process tree; test over-limit task. |
| SEC-05 | Evaluation tampering | Read-only trusted evaluator, hidden labels separate, content hashes; test forged output. |
| SEC-06 | Artifact traversal / malicious archive | Normalize and reject traversal/symlinks, enforce size/type, no unsafe deserialization. |
| SEC-07 | Unauthorized execution | Approval bound to hash and budget; deny unknown action; audit. |
| SEC-08 | Supply-chain risk | Pin image/dependency hashes, review licenses, scan dependencies; no automatic installs by model. |
| SEC-09 | Data leakage | Public datasets, redacted prompts/logs, no secrets in artifacts; scan exports. |
| SEC-10 | Unbounded autonomy | No self-modification, privilege escalation, open-ended replication or external posting; cancel and budget stop tests. |

## Runtime policy
Non-root user, read-only base filesystem, isolated temporary writable workspace, no network, bounded process count, timeout, resource quota, explicit artifact allowlist. Block host devices except explicitly approved GPU assignment on a dedicated worker. No remote execution by default. Model tool requests are structured data validated by trusted controller; no direct shell command from model. Approval required for each new task domain, cloud transfer, paid spend or policy change. Fail closed if policy verification unavailable.

## Response and disclosure
Cancel campaign; revoke worker credentials if applicable; quarantine artifacts; retain safe logs; assess incident; remediate and rerun regression tests; only then resume. Publish security contact and disclosure process before accepting outside contributions. No unsafe research domains; no autonomous publication. Respect dataset/model terms and third-party licenses, including redistribution restrictions.

## Rust boundary
Forbid first-party unsafe in core crates; audit dependencies and isolate FFI if later needed. Memory safety does not replace sandboxing. Python worker cannot receive provider credentials, hidden labels or ledger access. Do not invoke a shell with generated text; use validated executable and argument arrays. Bound output before parsing, reject symlinks and traversal, use isolated execution and explicit process-tree cleanup.
