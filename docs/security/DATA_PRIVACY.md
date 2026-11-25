# Data privacy and evidence retention

The initial release uses fixed synthetic fixture records. Later task-pack admission records the source version, source rights, collection constraints, redistribution decision, split identity, data owner and output rules. Review public datasets before loading them because availability does not establish permission or absence of personal information. Exclude personal and sensitive datasets from initial releases.

| Record | Access and retention |
| --- | --- |
| Public synthetic fixture and licensed inputs | Worker reads only its declared subset. Export includes files only when permission is documented. |
| Evaluator and held-out labels | Evaluator-only access before study closure. Record the version and digest in shared evidence. |
| Raw logs and optional prompts | Local access scoped to owner or reviewer. Exact prompts are retained only when safe and permitted. |
| Referenced evidence and failed attempts | Preserve the research and audit trail under its applicable retention policy. Export the redacted permitted subset. |
| Unreferenced transient artifacts | v0.2 proposes a 30-day grace period. Verify reachability and concurrent leases before collection. |
| Credential material | Remains outside datasets, manifests, logs, event payloads and export paths. |

Before release, classify and inspect artifacts, archives and logs, scan for synthetic credential canaries in tests, and verify the inventory and redistribution terms. For a sensitive artifact discovered after commit, stop export, quarantine it, record access and integrity impact, and append a redaction or invalidation event where evidence changed. Keep reviewable criteria for what a redacted bundle can reproduce. [Incident response](INCIDENT_RESPONSE.md)
