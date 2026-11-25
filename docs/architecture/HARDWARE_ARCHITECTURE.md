# Hardware architecture

The developer target is a Windows laptop with an Intel Core i5-13420H (8 cores, 12 logical processors) and NVIDIA RTX 3050 Laptop GPU with 6 GiB dedicated VRAM. Phase 1 is CPU-only. See [project status](../../PROJECT_STATUS.md) and the development environment profile for observed versions, RAM and free disk.

The initial reviewed worker profile proposes one job, two CPU threads and 2 GiB worker memory. Leave memory for the operating system and controller. Bound an attempt's output separately from cumulative disk retention. Document actual host, OS and runtime details for every evaluated campaign. A WSL2 distribution is a candidate Linux environment and still needs worker enforcement validation.

| Change | Evidence to collect |
|---|---|
| CPU first | Pinned compiler, native replay and locked dataset/code boundaries |
| GPU inference | Peak VRAM, numerical parity, warmup and throughput versus CPU |
| Remote compute | Worker acceptance, cost, stratified hardware observations and recovery |
| More concurrency | Saturation curve, resource ceilings, settlement and teardown |

Read-only GPU detection is independent of CUDA setup. Add a GPU toolchain only after reviewing workload and compatibility. See [source stack](../archive/v0.2/AETHER/07_TECH_STACK.md) and [resource limits](RESOURCE_MANAGEMENT.md).
