# DevOps strategy

Start with local, versioned documentation and a repeatable audit. After the first approved commit series, use a pinned Rust toolchain and application lockfile. Builds and tests target native Windows and supported Linux. Later isolated workers use separately pinned runtime images or environments.

Limit public release automation to vetted, read-only checks first. Use a distinct release job protected by owner approval and least privilege. Local development has no cloud dependency. [Pipeline design](CI_CD_DESIGN.md)
