# Repository structure

Root documents preserve mission, status, contribution rules, security contact process and release/license decisions. `docs/` contains reviewed architecture, research design, learning and audit material. `docs/archive/` preserves source bytes. Keep current decisions outside archives.

| Path | Purpose |
|---|---|
| crates/ | Native implementation crates activated one milestone at a time |
| learning/ | Standalone Cargo workspace for student-written exercises |
| contracts/ | Versioned JSON schemas and conformance examples |
| workers/python/ | Optional reviewed scientific workers in separate environments |
| taskpacks/ | Reviewed reproducible tasks and public fixture inputs |
| research/ | Literature, hypotheses, protocols, trials and release evidence |
| benchmarks/, tests/ | Benchmark assets and external system checks |
| infra/, scripts/, .github/ | Approved operator tooling and CI configuration |
| web/ | Future research workspace |

Documentation-only directories describe their activation criteria. Genesis has no active Cargo member or application entry point. Add lockfiles with the first runnable workspace. Keep hidden evaluation data, credentials, personal data, large raw artifacts and locally generated outputs outside tracked files.

See the [crate plan](../architecture/COMPONENT_ARCHITECTURE.md) and [source repository proposal](../archive/v0.2/AETHER/16_DEVOPS.md).
