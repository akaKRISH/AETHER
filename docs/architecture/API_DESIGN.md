# API design

The native CLI and single-owner workbench share the Rust execution components. Start the API with `aether-lab serve [port]`. It defaults to port 8080 on IPv4 loopback and serves the built web assets from the same origin. `GET /health` returns the service status and loopback scope. Every `/v1` route requires `Authorization: Bearer <AETHER_TOKEN>`. [Local startup](../operations/LOCAL_WORKBENCH.md)

## Implemented loopback API

In the table, `{id}` is the campaign ID. Send JSON bodies with `Content-Type: application/json`. The campaign ID is a lowercase ASCII identifier of 1–32 letters, digits or hyphens.

| Method and path | Request and response |
|---|---|
| GET /v1/runtime | Host OS, architecture, logical CPU count and reviewed worker profile |
| POST /v1/campaigns | [Plan request](../../taskpacks/example-campaign.json). Returns the frozen draft and protocol hash with 201. |
| GET /v1/campaigns | Bounded list. Optional `after` campaign ID and `limit` of 1–512, with a default of 128. |
| GET /v1/campaigns/{id} | Campaign snapshot with budget, claims, dependency graph, attempts and evidence |
| POST /v1/campaigns/{id}/approve | `{"protocol_hash":"<sha256>"}` approves the exact frozen document. Returns 409 if the digest or state conflicts. |
| POST /v1/campaigns/{id}/start | Returns 202 for an approved campaign and reserves the local supervisor slot. A conflicting state or busy slot returns 409. |
| POST /v1/campaigns/{id}/cancel | Requests cancellation and returns 202 |
| GET /v1/campaigns/{id}/runs | Runs and their execution attempts |
| GET /v1/campaigns/{id}/claims | Claims, dependencies and evidence references |
| GET /v1/campaigns/{id}/events | Up to 64 committed events after the optional numeric `after` sequence |
| GET /v1/campaigns/{id}/stream | Server-sent `campaign` events with sequence IDs. Resume with `Last-Event-ID` or `after`. |
| GET /v1/campaigns/{id}/report | Self-contained JSON report after digest checks |
| GET /v1/artifacts/{sha256} | Digest-checked raw bytes as a download |
| POST /v1/benchmarks | Matched synthetic default study and measured simulator wall time |
| POST /v1/proposals | [Planning request](../../taskpacks/example-planning.json). Returns reviewed-task proposals from the deterministic offline provider. |
| GET /v1/literature | Local citation metadata. Optional `query` filters the catalog. |
| POST /v1/literature/lookup | `{"query":"<search text>"}` requests Crossref metadata only when enabled explicitly. |

Application error bodies contain `code` and `error`. Typical statuses are 400 for invalid input, 401 for missing or incorrect credentials, 404 for missing resources, 409 for state or integrity conflicts, and 503 for internal operation failures. Router and JSON extractor rejections can also occur before an application handler runs. The loopback API uses one owner token across the local workspace. [HTTP implementation](../../crates/aether-api/src/http.rs) · [API tests](../../crates/aether-api/tests/platform.rs)

## Planned remote contract

The v0.2 hostable design introduces project-scoped authorization, TLS, request tracing and additional routes. This table preserves its design targets.

| Method and path | Contract |
|---|---|
| POST /v1/projects | Create, 201 |
| POST /v1/projects/{id}/protocols | Validate and freeze, 201 |
| POST /v1/protocols/{id}/approve | Approve exact config hash, 409 on mismatch |
| POST /v1/campaigns | Start approved protocol, 202, idempotency key required |
| GET /v1/campaigns/{id} | Status and remaining budget |
| POST /v1/campaigns/{id}/cancel | Idempotent request, 202 |
| GET /v1/campaigns/{id}/runs | Cursor-paginated attempts |
| GET /v1/campaigns/{id}/events | SSE after committed sequence |
| GET /v1/campaigns/{id}/claims | Current verdict and evidence references |
| GET /v1/artifacts/{sha256} | Authorized digest lookup, length and media type |
| POST /v1/campaigns/{id}/exports | Immutable export request, 202 |

Scope idempotency to caller, project and operation. A repeated key with the same request digest returns the prior acceptance. A conflicting body returns 409. Define bounded retention before release. Reauthorize reads, exports, actions and subscriptions. Errors expose `code`, `message` and `request_id`. Map malformed input to 400, unauthenticated to 401, denied to 403, missing to 404, invalid domain shape to 422, rate limit to 429 and unavailable to 503.

Verify project isolation, retry behavior, cursor replay and TRD-N05 metadata-read latency. See [source API](../archive/v0.2/AETHER/09_API_EVENTS.md).
