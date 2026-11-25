# Release criteria

- **R0:** M1 native fixture is accepted, reviewed baseline reruns in a clean environment, modes respect approval and denial rules, every quantitative assertion has provenance and restore tests pass. Use the verified exact fixture/data rights.
- **R1:** Pilot calibrates resources, required comparison arms pass the dev suite, held-out protocol is registered before opening test tasks, all allowed attempts are reported and at least one independent reproduction is documented. A negative effect is compatible with a scientifically valid release.
- **R2:** Demonstrated workflow need, owner authorization, observable and scoped API, accessible UI, bounded remote worker if shipped, and platform-specific failure and denial tests.
- **R3:** SDK contracts stable, authorized license and security channel, independent contributor path tested, artifact and report release reviewed.

Every published release includes a release note, supported targets, reproducibility instructions, licenses and known limits. Confirm test evidence in [quality gates](../testing/QUALITY_GATES.md).
