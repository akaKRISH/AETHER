# Model providers

The deterministic provider is available offline and uses zero model tokens. The browser proposal endpoint continues to use that provider. The model interface proposes a ranking of three reviewed sorting algorithms. It accepts an objective of at most 512 UTF-8 bytes and passes the original seed and item count through the trusted controller. A proposal has no execution or claim-verification authority. [Planning controller](../../crates/aether-api/src/planning.rs)

From the repository root, use a synthetic request to exercise the provider boundary.

```powershell
cargo run --locked --bin aether-lab -- propose-provider taskpacks/example-planning.json mock deterministic-v1
```

| Provider argument | Configuration and preflight |
|---|---|
| `mock` | Select `deterministic-v1` for the repeatable reference |
| `ollama` | Select an installed completion model by local name and tag. The adapter contacts loopback port 11434 and requires the daemon to confirm cloud is disabled. |
| `llama-cpp` | Select a simple model alias on an operator-configured local llama.cpp endpoint. Set `AETHER_LLAMA_CPP_PORT` to override loopback port 8081. |
| `openai` | Select an available model that supports Chat Completions and structured output. The adapter uses `OPENAI_API_KEY` and connects to the official HTTPS API. |
| `anthropic` | Select an available Claude model that supports Messages with structured output. The adapter uses `ANTHROPIC_API_KEY` and connects to the official HTTPS API. |

Hosted requests require prior approval for the specific transfer and spend. An approved operator sets `AETHER_ALLOW_EXTERNAL_MODELS=1` in the current process, supplies the matching credential in the environment and invokes the named provider. Use an objective cleared for that provider. The request includes the objective, seed, and item count. Hosted planning is an explicit standalone operation outside campaign budget accounting. It does not quote or report spend. [Human approval policy](../governance/HUMAN_APPROVAL_POLICY.md)

The adapters use separate bounded metadata and generation requests with redirects and environment proxies disabled. A metadata GET can be retried once after a short delay following a transient failure. Generation uses one request with a 25-second timeout and a 128-token output cap. Cancellation is checked between network stages and before accepting results. The synchronous transport may finish its in-flight request before cancellation takes effect. The batch response format keeps malformed output or unexpected tool calls outside the proposal list. [Provider implementation](../../crates/aether-api/src/planning/providers.rs)

The report includes the planning request digest, selected and reported model names, provider protocol, model digest when a valid Ollama digest is available, observed wall time, and provider-reported token counts when available. Missing counts are null. The CLI returns no credentials or raw model transcript. A model ID or digest is useful provenance, and repeated inference can still vary.

## API contracts checked

- Ollama uses structured generation and inspects the daemon's cloud policy before model metadata or inference. [Generate API](https://docs.ollama.com/api/generate) · [Cloud configuration](https://docs.ollama.com/faq#how-do-i-disable-ollamas-cloud-features) · [Upstream server routes](https://github.com/ollama/ollama/blob/main/server/routes.go)
- llama.cpp uses the model-list and Chat Completions paths with a JSON schema wrapper. [Server API](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md#openai-compatible-api-endpoints)
- OpenAI uses model retrieval and the Chat Completions JSON schema response format. [Model retrieval](https://developers.openai.com/api/reference/resources/models/methods/retrieve) · [Chat Completions](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create)
- Claude uses the Messages `output_config.format` field and Claude API version 2026-10-21. [Models](https://platform.claude.com/docs/en/api/models/retrieve) · [Messages](https://platform.claude.com/docs/en/api/messages/create) · [Structured outputs](https://platform.claude.com/docs/en/build-with-claude/structured-outputs)

## Validation scope

Only deterministic and fixture-based planning can be qualified without a running authorized model endpoint. At preflight, no local inference daemon was listening at the standard ports. An installed Qwen 3.5 4B manifest referenced approximately 2.47 GiB of weights and a 0.63 GiB projector. The host had 1.30 GiB of available system RAM at the initial observation, so model load and inference were left unqualified. Metadata and fixture runs do not produce a time-to-first-token or inference throughput measurement.
