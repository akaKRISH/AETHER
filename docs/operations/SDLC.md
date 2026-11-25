# Software development life cycle

Open one scoped issue with requirement ID, expected behavior and a failing case. Review the design and learning exercise, get owner approval, then implement the smallest slice. Run focused and regression checks, document the command and evidence, and update traceability before accepting the change.

A release candidate contains pinned dependencies, documented supported targets, a reproducible smoke command, known limits and no unresolved critical/high finding in shipped paths. Breaking changes need migration and rollback review. [Quality gates](../testing/QUALITY_GATES.md)
