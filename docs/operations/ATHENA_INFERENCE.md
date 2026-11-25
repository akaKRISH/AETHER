# Athena local inference qualification

## Host and process

The existing desktop service answered as Ollama 0.40.17 and reported cloud enabled. It had no resident models. The qualification started an owned daemon on a free loopback port with `OLLAMA_NO_CLOUD=1` and its own temporary model metadata. Its model blobs were hard linked from the installed copy. The daemon reported disabled cloud with source `env`. Its sole catalog entry matched the SHA-256 of the installed model manifest, and neither the model entry nor streamed responses advertised remote routing. Existing user services, configuration, and installed weights were preserved.

The model was Qwen 3.5 4B, reported as 4.2B parameters in GGUF Q4_K_M. Its installed descriptors totaled 3,324,175,492 bytes. The manifest digest was `e0b8c85c75aad2573ff49bec177ebb1013772fba665a53d64c04266439a2222d`. The host had 2.69 GiB free RAM and 42.43 GiB of disk at preflight. The RTX 3050 driver was 610.91 with 6,144 MiB total VRAM and 5,911 MiB free. No model download or system software change was needed. [Official local-only configuration](https://docs.ollama.com/faq#how-do-i-disable-ollamas-cloud-features)

## First calibration

The harness used a fixed prompt, seed 19, temperature zero, thinking disabled, and a 32-token output cap. Each arm ran one cold request, one warmup, and three measured warm requests.

| Requested context | Reported resident context | Cold load | Cold first text | Warm first text median | Warm generation median |
|---|---:|---:|---:|---:|---:|
| 512 | 2,048 | 14.09 s | 14.85 s | 167 ms | 31.53 tokens/s |
| 2,048 | 2,048 | 13.81 s | 14.37 s | 189 ms | 14.03 tokens/s |

The server used 2,048 resident tokens in both arms, so these two rows measure repeated behavior at one effective context. NVML samples peaked at 3,753 MiB used device-wide. Ollama reported 3,071,289,588 bytes of model GPU residency. The monitor observed a minimum of 1,664,008,192 bytes of free system memory, and aggregate host CPU occupancy during the measurement was 49.24%. Device usage includes other processes. Cached prompt timings are retained in the raw counters. [Calibration export](../../research/results/athena-ollama-inference.json)

## Matched context follow-up

The follow-up used the same installed model, prompt and generation settings. Ollama confirmed that the requested and resident context lengths matched in these trials.

| Resident context | Cold load | Cold first text | Cold prompt tokens/s | Median warm first text | Median warm generation | Median warm total |
|---|---:|---:|---:|---:|---:|---:|
| 2,048 | 15.15 s | 16.02 s | 30.36 | 232 ms | 13.49 tokens/s | 2.58 s |
| 4,096 | 15.56 s | 16.49 s | 26.83 | 224 ms | 17.63 tokens/s | 2.02 s |

The reported GPU residency was 3,071,289,588 bytes at 2,048 and 3,140,495,604 bytes at 4,096. Device-wide sampled peak was 3,775 MiB, and sampled free host memory fell to 1,730,420,736 bytes. Host-wide CPU occupancy was 53.77%. The three warm samples per context leave the speed comparison descriptive. The same short prompt was used in both arms, so the probe varies allocated context capacity. CPU-only and alternate-quantization comparisons remain unmeasured. [Matched-context raw counters and samples](../../research/results/athena-ollama-contexts.json)

## Native inference options

Candle has Rust GGUF and CUDA examples and active Qwen 3.5 implementation work. These references do not establish parity with the installed Q4_K_M artifact, chat template, response schema or Windows GPU build. Burn documents CUDA and CPU backends and an ONNX-to-Rust import path. That path requires an independently verified model and operator match. Athena keeps the already-qualified Ollama runtime. An equivalent native comparison would pin weights and tokenization, confirm schema and output agreement, and measure the same cold and warm timing boundaries. [Candle](https://github.com/huggingface/candle) · [Candle Qwen 3.5 implementation discussion](https://github.com/huggingface/candle/issues/3837) · [Burn backends](https://burn.dev/docs/burn/) · [Burn ONNX import](https://burn.dev/books/burn/onnx-import.html)

## Campaign procedure

`python scripts/athena_ollama.py preflight` checks the installed model and owned server before any project prompt. The `benchmark` command repeats the calibration. The `draft` command creates a cited objective draft with the chosen model via the release AETHER CLI. Model-specific settings and the canonical prompt digest are frozen in the report, and inference usage is separate from the later worker budget. Keep the exact-built executable through approval and execution.

The first objective-planning call reported 127 input and 64 output tokens, with 15,958 ms measured by the adapter. The selected candidate was insertion sort. The operator-proposed objective asks the trusted verifier to compare its exact output against a reference on the original and distinct challenge inputs and to reproduce the original in a new process. The model's speculation about the other alternatives is outside this protocol. The stored Crossref record is selected background metadata. The draft is waiting for explicit owner approval. [Frozen draft](../../research/results/athena-local-draft-report.json) · [Ollama generate contract](https://docs.ollama.com/api/generate) · [Structured outputs](https://docs.ollama.com/capabilities/structured-outputs)
