# Development policy

Begin each approved milestone with one narrow learning objective and a measurable engineering result. Identify the task’s v0.2 source, requirements and likely failure cases. Explain the underlying concept, alternatives and chosen approach. Ask the owner to work a small exercise and explain the trade-off before implementation approval.

Use small reviewable changes with the protocol and contract versions recorded as needed. A code change references the requirement, describes its behavior, and includes a targeted verification method. Log a failed check honestly along with its user or research impact. Rust code should use bounded, typed inputs and preserve transaction and recovery invariants. Dependencies require a purpose, compatibility, security and license review.

A proposed architectural change starts with an ADR and updates the affected design, tests and documentation. Work that changes privileges, introduces a new task domain, shifts resources externally or publishes evidence goes through [human approval](HUMAN_APPROVAL_POLICY.md). State the supported platform and the component tested when making a readiness claim.

Onboarding order is the [agent instructions](../../AGENTS.md), the [project context](../../PROJECT_CONTEXT.md), the [decision register](ARCHITECTURE_DECISION_RECORDS.md) and the specific requirement under review.
