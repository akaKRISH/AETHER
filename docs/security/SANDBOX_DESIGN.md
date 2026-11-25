# Sandbox design

The first native fixture uses built-in, reviewed Rust and synthetic records, with no subprocess and no isolation claim. A later reviewed worker uses a Linux-first supervisor. Treat WSL2 as a development environment whose configuration needs testing before worker use. A container is a packaging and defense-in-depth layer. Use a dedicated VM or hardened isolated environment for any future authorized workload that requires a stronger boundary. Arbitrary generated executable code on personal hardware is outside scope.

A qualified worker profile requires a dedicated unprivileged execution identity, read-only code and inputs, a fresh bounded output workspace, default-denied egress, no inherited credentials or host devices, and an OS-enforced resource envelope. Quota scope covers descendants. Use a monotonic execution deadline, cancellation followed by bounded cleanup, and a durable record of uncertain teardown. Prevent resume until the supervisor confirms cleanup or reconciles the attempt. The controller must refuse dispatch when the selected environment cannot prove it can apply the profile.

Ingest after execution has stopped. Resolve manifest entries against the output root using OS-aware path handling that rejects links and reparse points, enforce file type and count limits and hash copied bytes into private storage. Prefer a single trusted ingestion path that avoids check-then-use races. Report an artifact failure without publishing worker-controlled diagnostic content.

Qualify the runner with benign negative tests for resource limits, egress, denied host access and process-tree cancellation on the actual execution profile. Record OS and configuration in results. [Threat model](THREAT_MODEL.md)
