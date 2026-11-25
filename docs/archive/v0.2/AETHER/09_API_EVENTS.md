# 09 | API, event and extension contracts

## R2 proposed HTTP API (R0 uses equivalent CLI service)
| Method / path | Behavior |
|---|---|
| POST /v1/projects | Create project; 201. |
| POST /v1/projects/{id}/protocols | Validate and freeze config; 201. |
| POST /v1/protocols/{id}/approve | Body contains config_hash; 409 on mismatch. |
| POST /v1/campaigns | Start approved protocol; 202; Idempotency-Key required. |
| GET /v1/campaigns/{id} | Status and remaining budget; 200. |
| POST /v1/campaigns/{id}/cancel | Idempotent cancellation request; 202. |
| GET /v1/campaigns/{id}/runs | Cursor-paginated attempts; 200. |
| GET /v1/campaigns/{id}/events | SSE with sequence cursor, replay after reconnect. |
| GET /v1/campaigns/{id}/claims | Predicates, verdicts and evidence references. |
| GET /v1/artifacts/{sha256} | Authorized download with size/type headers. |
| POST /v1/campaigns/{id}/exports | Generate immutable bundle; 202. |

Errors: 400 malformed request, 401 unauthenticated, 403 unauthorized/policy violation, 404 missing resource, 409 state/hash conflict, 422 schema invalid, 429 rate limit, 503 unavailable. Return `{code,message,request_id}` with no secrets. API bound to loopback in local mode; remote access requires TLS and authentication. Every resource access must be scoped to project membership when multi-user is introduced.

## Event contract
Required: schema_version, event_id, campaign_id, sequence, type, timestamp, actor, correlation_id, payload. Types include protocol.approved, budget.reserved, run.leased, run.started, run.completed, run.failed, budget.settled, claim.proposed, claim.verified, claim.rejected, claim.revoked, campaign.cancelled. Delivery at least once; consumers deduplicate event_id and resume from sequence. Unknown version fails safe. Model messages are proposals, not events that directly change trusted state.

## Task adapter contract
A task pack exposes metadata and license, pinned input digest, candidate template, resource profile, allowed operations, public development cases, trusted evaluator, hidden test hash and metric directions. Candidate returns predictions or declared artifacts, not final verdicts. Evaluator executes outside candidate context; a task adapter cannot grant itself policy permissions. JSON drafts in `contracts/` are examples, not running APIs.

## Worker protocol v1
See contracts/worker-job.schema.json and worker-result.schema.json. Input and output are single JSON files in a controlled workspace, not arbitrary commands. Worker returns predictions/artifact references; only Rust controller records events. Reject unknown schema versions, mismatched attempt IDs and oversized or invalid output. API service later uses Axum and SSE.
