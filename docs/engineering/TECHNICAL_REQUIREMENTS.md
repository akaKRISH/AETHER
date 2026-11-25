# Technical requirements

The IDs retain the v0.2 authority. All behaviors below are proposed and gain runtime evidence at the listed verification gate.

| ID | Measurable behavior | Verification |
|---|---|---|
| TRD-N01 | CPU local path without GPU, one reviewed worker by default | T03 fixture, reviewed-worker CPU smoke |
| TRD-N02 | Atomic reservations and settlement. No new admission beyond caps. | T04 concurrent reserve and repeat settlement |
| TRD-N03 | Initiate cancellation within 2 seconds and terminate CPU descendant processes within 10 seconds on qualified runtime | T09 timed process-tree fixture |
| TRD-N04 | Verify hashes at ingest and replay and block verified export on tamper | T05 mutate one byte |
| TRD-N05 | R2 metadata-read p95 ≤ 500 ms under 5 clients, excluding worker/model work | R2 recorded load test |
| TRD-N06 | Candidate cannot access network, secrets, host mounts or container control socket | T10 denial tests |
| TRD-N07 | At-least-once work cannot produce two committed results for an attempt | T08 interruption and recovery |
| TRD-N08 | Local backup target RPO 24 hours and RTO 2 hours | T12 timed restore drill |
| TRD-N09 | Every attempt reports structured start/end, usage and stable error class when failed | T03/T08 schema and exceptional paths |
| TRD-N10 | R2 keyboard access and WCAG 2.2 AA target | T14 manual and automated checks |
| TRD-N11 | Rust control plane and fixture build/run in an environment without Python | T17 |
| TRD-N12 | Versioned worker envelopes and denied direct worker writes to trusted state | T18/T19 |
| TRD-N13 | Cross-process cancellation, bounded output and recovery | T18/T21 and quota fixture |
| TRD-N14 | First-party core crates forbid unsafe code | T20 workspace lint plus dependency review |

See [source SRS](../archive/v0.2/AETHER/04_TRD_SRS.md) and [requirements traceability](../testing/REQUIREMENTS_TRACEABILITY.md).
