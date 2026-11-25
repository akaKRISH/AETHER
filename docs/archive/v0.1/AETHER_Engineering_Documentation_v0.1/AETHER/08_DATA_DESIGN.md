# 08 | Data architecture and evidence model

## Logical schema (primary keys shown as id)
| Entity | Essential fields / relationships |
|---|---|
| project | id, name, created_at |
| protocol | id, project_id, version, config_hash UNIQUE, config_json, approved_at |
| campaign | id, protocol_id, status, started_at, finished_at |
| budget | campaign_id PK, caps_json, used_json, reserved_json, version |
| run | id, campaign_id, parent_run_id nullable, action, seed, status, lease_token, code_hash |
| artifact | sha256 PK, size_bytes, media_type, storage_key, created_at |
| run_artifact | run_id, sha256, role; composite key |
| metric | id, run_id, name, value, unit, evaluator_hash, split_id |
| claim | id, campaign_id, predicate_json, status, revision, created_at |
| evidence_edge | id, claim_id, artifact_hash or metric_id, relation, required, run_id |
| verification | id, claim_id, replay_run_id, verdict, rule_hash, reason |
| event | id, campaign_id, sequence UNIQUE within campaign, kind, payload_json, occurred_at |

Foreign keys enabled. Index run(campaign_id,status), event(campaign_id,sequence), evidence_edge(claim_id), metric(run_id,name). Exactly one evidence target per edge, checked by constraint. Store canonical JSON with stable key order and version; reject non-finite floats. Use UTC timestamps and monotonic clock for elapsed times. Artifact storage path derived solely from hash, never from model-supplied paths. Event audit is append-only by application convention, not cryptographically tamper-proof; export signed manifests only after key management design.

## Evidence semantics
Relations: SUPPORTS, CONTRADICTS, DEPENDS_ON, REPRODUCES. A claim graph is a DAG of claim dependencies plus evidence links; cycle insertion rejected. Required links and evaluator rules are frozen in protocol. A claim may have negative evidence without being automatically rejected if the rule allows conflicting data; verifier records explicit reason. Raw metrics are distinct from derived claims; unsupported model text never becomes a metric.

## Artifact manifest and retention
Manifest records task/dataset hash, split hash, code hash, dependencies/image digest, seeds, runtime hardware, evaluator version, policy, model identifier and prompt hash if used, stdout/stderr digest, resource usage, result and errors. Keep verified bundles and failed trials for research audit. Local default: 30-day retention for transient unreferenced blobs; never garbage collect referenced data. Use public licensed datasets; do not store credentials, personal data or model secrets. Export redacted logs; retain exact prompts only when safe and permitted.
