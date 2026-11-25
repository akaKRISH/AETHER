# Memory management

Use borrowed slices for fixture labels and predictions where the caller keeps ownership. Compute exact accuracy counts in a single pass and avoid cloning inputs. Store owned immutable digests, configuration and IDs where data outlive the call. Prefault and limit input construction at trust boundaries rather than trusting reported lengths.

| Allocation site | Control |
|---|---|
| Built-in fixture | Fixed-size source data and borrowed read-only evaluation |
| JSON parser | Bound bytes/depth and reject oversize before parsing |
| Artifact hashes | Streaming reads with a bounded buffer |
| Logs and process pipes | Drain with strict size and time ceiling |
| Graph projection | Load campaign-scoped graph with a measured node/edge limit |
| Worker RAM and GPU VRAM | Supervisor-enforced worker profile and telemetry |

Rust's ownership controls memory lifetime inside a program. It does not constrain an external process or protect against unchecked allocations from large inputs. Measure peak resident memory separately from cumulative allocation and GPU VRAM. Error and cancellation paths close streams and release temporary buffers. Restore fixtures include oversized input and intentional worker exhaustion.

See [hardware profile](../architecture/HARDWARE_ARCHITECTURE.md).
