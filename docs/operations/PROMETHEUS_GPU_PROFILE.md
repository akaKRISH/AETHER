# Prometheus GPU profile

The supported development host reported an NVIDIA GeForce RTX 3050 Laptop GPU with compute capability 8.6, driver 610.91, and 6,144 MiB of dedicated VRAM. The preflight snapshot showed about 5,880 MiB free GPU memory, 1.30 GiB free host RAM out of 15.64 GiB, and 47.41 GiB free disk. These are observations at the time of the probes.

`aether-lab hardware` and the Runtime panel query the installed NVIDIA driver tool with fixed arguments, a bounded stdout pipe and a two-second timeout. The Windows child receives only `SystemRoot` and `ProgramFiles`. The latter was required for NVML initialization on this host. An absent or invalid reading produces an explicit unavailable status. [Observed driver output](../../research/results/prometheus-gpu-observation.json) · [Rust diagnostic](../../crates/aether-api/src/hardware.rs)

The driver reported CUDA UMD 13.3. The toolkit compiler `nvcc` was absent from PATH. Driver compatibility describes a supported CUDA API level. CUDA build and inference qualification require an actual compatible toolkit or a matched prebuilt runtime. [NVIDIA compatibility matrix](https://docs.nvidia.com/datacenter/tesla/drivers/latest/cuda-toolkit-driver-and-architecture-matrix.html)

| Option | What the adapter would need |
|---|---|
| Candle | Its CUDA build instructions require a working CUDA toolchain and a known compute capability. [Candle installation](https://huggingface.github.io/candle/guide/installation.html) |
| Burn | An explicit backend and build feature plus a supported device. [Burn backends](https://burn.dev/books/burn/building-blocks/backend.html) |
| ONNX Runtime | The selected CUDA provider build and matching CUDA/cuDNN dependencies. [CUDA provider requirements](https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html) |
| PyTorch | A compatible package and approved isolated worker contract. [PyTorch platform packages](https://pytorch.org/get-started/locally/) |

The initial local inference check found no listener. Later checks reached an Ollama daemon whose cloud-enabled policy prevented its use for approved local inference. The installed `qwen3.5:4b` weight and projector layers total about 3.10 GiB before runtime allocations. Host free memory was low during qualification. No model was loaded and no toolkit or model was downloaded.

When an authorized local endpoint and sufficient headroom are available, freeze one existing model digest, quantization, context size, prompt file, seed and output cap. Capture cold load, first emitted token, generation end, emitted token count, process CPU and peak GPU memory as distinct observations. Run one warmup and repeated warm requests with identical inputs, and report elapsed distribution, tokens/s and model configuration. These inference measurements remain open. [Provider status](MODEL_PROVIDERS.md)
