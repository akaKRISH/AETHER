# Non-functional requirements

[Technical authority](../archive/v0.2/AETHER/04_TRD_SRS.md) assigns the TRD-N IDs. Keep technical thresholds in that requirement family and [traceability](../testing/REQUIREMENTS_TRACEABILITY.md).

- **Reliability:** Record work, failures, replay and actual resource use. Crash recovery must preserve consistency and avoid duplicate terminal effects.
- **Safety:** Limit execution to approved tasks, isolate reviewed external processes, and fail closed at every trust boundary. [Threat model](../security/THREAT_MODEL.md)
- **Reproducibility:** Protect artifacts with hashes, pin versions, and retain every eligible attempt.
- **Portability:** Demonstrate the native CPU path without Python, GPU or external API. Qualify process isolation per host before advertising worker support.
- **Usability:** Expose actionable errors and readable evidence. Later web accessibility targets WCAG 2.2 AA and keyboard operation. [W3C WCAG 2.2](https://www.w3.org/TR/WCAG22/)
- **Operability:** Target local RPO of 24 hours and RTO of 2 hours. Validate targets through restore drills on supported environments.
