# Dependency inventory

## Athena lock review

| Lock | SHA-256 | Observed advisory result |
|---|---|---|
| Cargo | `dcf4a1b6e5b1c79339131558f825b6dd725a28798bece976383d8fba2e314d4e` | RustSec 0.23.0 checked 202 lock entries against 1,568 advisories. Exit 0, zero findings and no warnings. |
| npm | `33b523e51e635e441b6a5e1b03f32592d26128563159ccd93cdf5369cc3fdccb` | npm's lock-only audit checked 82 entries. Exit 0 and zero findings. |

The Windows-only runtime addition is pinned `win32job` 2.0.3 (MIT or Apache-2.0), plus its Windows bindings and supporting dependencies. The diagnostic example reuses the workspace's existing `serde_json` as a development dependency. The RustSec check used the official Windows release archive with SHA-256 `aafeb61206730029601a72e669a12231f8674427e6248d02f6cb382cb36d56b4`. [Cargo lock](../../Cargo.lock) · [RustSec snapshot](https://github.com/RustSec/advisory-db/commit/b9b8a3c2cb1f2a3c76ab17a551193568c9f8b7a4) · [Containment decision](../security/ATHENA_WORKER_CONTAINMENT.md)

## Prometheus lock updates

| Lock | SHA-256 | Latest advisory result |
|---|---|---|
| Cargo | `d9ab513ddb54407b02b26392eecec8b49038de32dc6eb1566ca771c21bbbab7c` | RustSec 0.23.0 scanned 188 entries against 1,568 advisories. Exit 0, with zero findings and no warnings. |
| npm | `33b523e51e635e441b6a5e1b03f32592d26128563159ccd93cdf5369cc3fdccb` | npm 12.0.2 scanned 82 reported dependency entries. Exit 0, with zero findings. |

The RustSec run verified the official Windows tool download digest and used advisory database revision `b9b8a3c2cb1f2a3c76ab17a551193568c9f8b7a4`. The npm audit used `--package-lock-only`. These results cover known advisories for the recorded locks. [RustSec snapshot](https://github.com/RustSec/advisory-db/commit/b9b8a3c2cb1f2a3c76ab17a551193568c9f8b7a4) · [npm audit](https://docs.npmjs.com/cli/commands/npm-audit/)

Direct additions are `roxmltree` 0.21.2 for bounded Atom metadata parsing, Criterion 0.8.2 for two development-only benchmark targets, and Biome 2.6.9 for frontend development linting. Each publishes a permissive MIT or Apache-2.0 choice. The full updated lockfiles include the related transitive and platform entries. Recheck exact notice requirements before distribution. [Cargo lock](../../Cargo.lock) · [npm lock](../../web/package-lock.json)

## Foundation inventory

**Snapshot dated 25 November 2026.** This inventory covers the complete native [Cargo lock](../../Cargo.lock) and [web npm lock](../../web/package-lock.json), including transitive, development, optional and platform-specific entries. A lock can contain dependencies for several targets. Only the selected dependency graph reaches a particular build.

| Lock | SHA-256 | Scope |
| --- | --- | --- |
| Cargo | `26324c1e73fece499ebf625615bc11613ed223f43c24ed7971115a64fd25bde3` | 147 crates.io entries and 6 workspace crates |
| npm | `2a3aa107f37102fe85f0626ed888cb110aab8d7b232e4077be64111827adbcdb` | 71 external lock entries, registry hosts registry.npmjs.org |

The Cargo inventory matched every name, version and source in `cargo metadata --locked --format-version 1` to its lock entry, then read licenses from declared package metadata. npm names, versions and licenses are from the lock's package records, including optional build platform packages. Declared expressions are preserved as published, including legacy slash expressions. A dependency is classified as direct if a workspace member declares it. Consult the exact build feature and target graph for release notices. Neither manifest metadata nor advisory tools establish the rights to distribute project content.

## Advisory checks

| Check | Result |
| --- | --- |
| RustSec cargo-audit 0.23.0, original Cargo lock | Exit 0. Matched 153 lock entries against 1,567 advisories. No vulnerabilities or informational warnings reported. |
| npm 12.0.2 using registry.npmjs.org, web lock | Exit 0. The registry reported zero advisories across 71 locked dependency entries. Its report included development and optional packages. |

Cargo Audit used a portable Windows release ZIP whose digest was verified against GitHub release metadata. The local advisory database was pinned to commit `10db6998b5d74ec438bcf0763125bd425a068a8f`. The command audited the project lock with `audit --db <local-advisory-db> --no-fetch --json`. npm ran `npm --prefix web audit --package-lock-only --json`. Advisory checks cover known issues matched by the databases at scan time. [RustSec Cargo Audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit) · [RustSec snapshot](https://github.com/RustSec/advisory-db/commit/10db6998b5d74ec438bcf0763125bd425a068a8f) · [npm audit](https://docs.npmjs.com/cli/commands/npm-audit/)

## License counts

| Cargo license expression | Registry packages |
| --- | ---: |
| MIT OR Apache-2.0 | 76 |
| MIT | 24 |
| Unicode-3.0 | 18 |
| Apache-2.0 OR MIT | 8 |
| MIT/Apache-2.0 | 5 |
| ISC | 2 |
| Zlib | 2 |
| (MIT OR Apache-2.0) AND Unicode-3.0 | 1 |
| 0BSD OR MIT OR Apache-2.0 | 1 |
| Apache-2.0 | 1 |
| Apache-2.0 AND ISC | 1 |
| Apache-2.0 OR BSL-1.0 | 1 |
| Apache-2.0 OR ISC OR MIT | 1 |
| Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 1 |
| BSD-3-Clause | 1 |
| CDLA-Permissive-2.0 | 1 |
| MIT AND BSD-3-Clause | 1 |
| MIT OR Zlib OR Apache-2.0 | 1 |
| Unlicense OR MIT | 1 |

| npm license declaration | External entries |
| --- | ---: |
| MIT | 68 |
| Apache-2.0 | 1 |
| BSD-3-Clause | 1 |
| ISC | 1 |

The six unpublished workspace crate manifests do not yet declare a first-party license. The web package is private and its release terms are pending. Before distribution, review actual license and notice files for the exact shipped graph and decide which alternative expressions apply. Independently review the archival documents, datasets, media, and tool or model artifacts. [License decision](../../LICENSE_DECISION.md) · [License evaluation](LICENSE_EVALUATION.md)

`libsqlite3-sys 0.40.0` contains the SQLite C amalgamation, version 3.54.1. Its SHA3-256 `e2b869d3003b33c942b7dd7063604fbc53553e1b66609570df39b9b7b558ee81` matches the upstream release record. The amalgamation's public-domain dedication is separate from the Rust binding package's declared license. The advisory tools are lock-level checks and do not scan the installed operating system or certify the application binary. [SQLite release history](https://sqlite.org/changes.html) · [SQLite copyright](https://sqlite.org/copyright.html)

## Cargo entries

| Package | Locked version | Role | Declared license |
| --- | --- | --- | --- |
| `adler2` | `2.0.1` | Transitive registry | 0BSD OR MIT OR Apache-2.0 |
| `aether-api` | `0.1.0` | Workspace | Unspecified |
| `aether-cli` | `0.1.0` | Workspace | Unspecified |
| `aether-core` | `0.1.0` | Workspace | Unspecified |
| `aether-engine` | `0.1.0` | Workspace | Unspecified |
| `aether-runner` | `0.1.0` | Workspace | Unspecified |
| `aether-store` | `0.1.0` | Workspace | Unspecified |
| `atomic-waker` | `1.1.2` | Transitive registry | Apache-2.0 OR MIT |
| `axum` | `0.8.9` | Direct registry | MIT |
| `axum-core` | `0.5.6` | Transitive registry | MIT |
| `base64` | `0.23.1` | Transitive registry | MIT OR Apache-2.0 |
| `bitflags` | `2.13.2` | Transitive registry | MIT OR Apache-2.0 |
| `block-buffer` | `0.10.4` | Transitive registry | MIT OR Apache-2.0 |
| `bytes` | `1.12.2` | Transitive registry | MIT |
| `cc` | `1.11.3` | Transitive registry | MIT OR Apache-2.0 |
| `cfg-if` | `1.0.5` | Transitive registry | MIT OR Apache-2.0 |
| `cookie` | `0.18.3` | Transitive registry | MIT OR Apache-2.0 |
| `cookie_store` | `0.22.1` | Transitive registry | MIT OR Apache-2.0 |
| `cpufeatures` | `0.2.17` | Transitive registry | MIT OR Apache-2.0 |
| `crc32fast` | `1.5.6` | Transitive registry | MIT OR Apache-2.0 |
| `crypto-common` | `0.1.7` | Transitive registry | MIT OR Apache-2.0 |
| `deranged` | `0.5.9` | Transitive registry | MIT OR Apache-2.0 |
| `digest` | `0.10.7` | Transitive registry | MIT OR Apache-2.0 |
| `displaydoc` | `0.2.7` | Transitive registry | MIT OR Apache-2.0 |
| `document-features` | `0.2.12` | Transitive registry | MIT OR Apache-2.0 |
| `equivalent` | `1.0.3` | Transitive registry | Apache-2.0 OR MIT |
| `errno` | `0.3.15` | Transitive registry | MIT OR Apache-2.0 |
| `fallible-iterator` | `0.3.0` | Transitive registry | MIT/Apache-2.0 |
| `fallible-streaming-iterator` | `0.1.9` | Transitive registry | MIT/Apache-2.0 |
| `find-msvc-tools` | `0.1.15` | Transitive registry | MIT OR Apache-2.0 |
| `flate2` | `1.1.11` | Transitive registry | MIT OR Apache-2.0 |
| `foldhash` | `0.2.0` | Transitive registry | Zlib |
| `form_urlencoded` | `1.2.3` | Transitive registry | MIT OR Apache-2.0 |
| `futures-core` | `0.3.34` | Transitive registry | MIT OR Apache-2.0 |
| `futures-macro` | `0.3.34` | Transitive registry | MIT OR Apache-2.0 |
| `futures-task` | `0.3.34` | Transitive registry | MIT OR Apache-2.0 |
| `futures-util` | `0.3.34` | Direct registry | MIT OR Apache-2.0 |
| `generic-array` | `0.14.7` | Transitive registry | MIT |
| `getrandom` | `0.2.17` | Transitive registry | MIT OR Apache-2.0 |
| `hashbrown` | `0.17.2` | Transitive registry | MIT OR Apache-2.0 |
| `hashlink` | `0.12.2` | Transitive registry | MIT OR Apache-2.0 |
| `http` | `1.5.0` | Transitive registry | MIT OR Apache-2.0 |
| `http-body` | `1.1.0` | Transitive registry | MIT |
| `http-body-util` | `0.1.5` | Transitive registry | MIT |
| `httparse` | `1.10.1` | Transitive registry | MIT OR Apache-2.0 |
| `httpdate` | `1.0.3` | Transitive registry | MIT OR Apache-2.0 |
| `hyper` | `1.13.0` | Transitive registry | MIT |
| `hyper-util` | `0.1.21` | Transitive registry | MIT |
| `icu_collections` | `2.4.0` | Transitive registry | Unicode-3.0 |
| `icu_locale_core` | `2.4.0` | Transitive registry | Unicode-3.0 |
| `icu_normalizer` | `2.4.0` | Transitive registry | Unicode-3.0 |
| `icu_normalizer_data` | `2.4.0` | Transitive registry | Unicode-3.0 |
| `icu_properties` | `2.4.0` | Transitive registry | Unicode-3.0 |
| `icu_properties_data` | `2.4.0` | Transitive registry | Unicode-3.0 |
| `icu_provider` | `2.4.0` | Transitive registry | Unicode-3.0 |
| `idna` | `1.1.0` | Transitive registry | MIT OR Apache-2.0 |
| `idna_adapter` | `1.2.2` | Transitive registry | Apache-2.0 OR MIT |
| `indexmap` | `2.14.2` | Transitive registry | Apache-2.0 OR MIT |
| `itoa` | `1.0.19` | Transitive registry | MIT OR Apache-2.0 |
| `libc` | `0.2.191` | Transitive registry | MIT OR Apache-2.0 |
| `libsqlite3-sys` | `0.40.0` | Transitive registry | MIT |
| `litemap` | `0.8.3` | Transitive registry | Unicode-3.0 |
| `litrs` | `1.1.0` | Transitive registry | MIT OR Apache-2.0 |
| `log` | `0.4.35` | Transitive registry | MIT OR Apache-2.0 |
| `matchit` | `0.8.4` | Transitive registry | MIT AND BSD-3-Clause |
| `memchr` | `2.8.3` | Transitive registry | Unlicense OR MIT |
| `mime` | `0.3.18` | Transitive registry | MIT OR Apache-2.0 |
| `miniz_oxide` | `0.9.1` | Transitive registry | MIT OR Zlib OR Apache-2.0 |
| `mio` | `1.2.5` | Transitive registry | MIT |
| `num-conv` | `0.2.2` | Transitive registry | MIT OR Apache-2.0 |
| `once_cell` | `1.21.4` | Transitive registry | MIT OR Apache-2.0 |
| `percent-encoding` | `2.3.2` | Transitive registry | MIT OR Apache-2.0 |
| `pin-project-lite` | `0.2.18` | Transitive registry | Apache-2.0 OR MIT |
| `pkg-config` | `0.3.34` | Transitive registry | MIT OR Apache-2.0 |
| `potential_utf` | `0.1.6` | Transitive registry | Unicode-3.0 |
| `powerfmt` | `0.2.1` | Transitive registry | MIT OR Apache-2.0 |
| `proc-macro2` | `1.0.107` | Transitive registry | MIT OR Apache-2.0 |
| `quote` | `1.0.47` | Transitive registry | MIT OR Apache-2.0 |
| `ring` | `0.17.14` | Transitive registry | Apache-2.0 AND ISC |
| `rsqlite-vfs` | `0.2.0` | Transitive registry | MIT |
| `rusqlite` | `0.43.0` | Direct registry | MIT |
| `rustls` | `0.23.46` | Transitive registry | Apache-2.0 OR ISC OR MIT |
| `rustls-pki-types` | `1.16.0` | Transitive registry | MIT OR Apache-2.0 |
| `rustls-webpki` | `0.103.17` | Transitive registry | ISC |
| `ryu` | `1.0.23` | Transitive registry | Apache-2.0 OR BSL-1.0 |
| `serde` | `1.0.229` | Direct registry | MIT OR Apache-2.0 |
| `serde_core` | `1.0.229` | Transitive registry | MIT OR Apache-2.0 |
| `serde_derive` | `1.0.229` | Transitive registry | MIT OR Apache-2.0 |
| `serde_json` | `1.0.154` | Direct registry | MIT OR Apache-2.0 |
| `serde_path_to_error` | `0.1.20` | Transitive registry | MIT OR Apache-2.0 |
| `serde_urlencoded` | `0.7.1` | Transitive registry | MIT/Apache-2.0 |
| `sha2` | `0.10.9` | Direct registry | MIT OR Apache-2.0 |
| `shlex` | `2.0.1` | Transitive registry | MIT OR Apache-2.0 |
| `signal-hook-registry` | `1.4.8` | Transitive registry | MIT OR Apache-2.0 |
| `simd-adler32` | `0.3.10` | Transitive registry | MIT |
| `slab` | `0.4.12` | Transitive registry | MIT |
| `smallvec` | `1.16.6` | Transitive registry | MIT OR Apache-2.0 |
| `socket2` | `0.6.5` | Transitive registry | MIT OR Apache-2.0 |
| `sqlite-wasm-rs` | `0.6.3` | Transitive registry | MIT |
| `stable_deref_trait` | `1.2.1` | Transitive registry | MIT OR Apache-2.0 |
| `subtle` | `2.6.1` | Transitive registry | BSD-3-Clause |
| `syn` | `3.0.10` | Transitive registry | MIT OR Apache-2.0 |
| `sync_wrapper` | `1.0.2` | Transitive registry | Apache-2.0 |
| `synstructure` | `0.14.0` | Transitive registry | MIT |
| `thiserror` | `2.0.21` | Direct registry | MIT OR Apache-2.0 |
| `thiserror-impl` | `2.0.21` | Transitive registry | MIT OR Apache-2.0 |
| `time` | `0.3.55` | Transitive registry | MIT OR Apache-2.0 |
| `time-core` | `0.1.9` | Transitive registry | MIT OR Apache-2.0 |
| `time-macros` | `0.2.32` | Transitive registry | MIT OR Apache-2.0 |
| `tinystr` | `0.8.5` | Transitive registry | Unicode-3.0 |
| `tokio` | `1.54.1` | Direct registry | MIT |
| `tokio-macros` | `2.7.2` | Transitive registry | MIT |
| `tower` | `0.5.3` | Transitive registry | MIT |
| `tower-layer` | `0.3.3` | Transitive registry | MIT |
| `tower-service` | `0.3.3` | Transitive registry | MIT |
| `tracing` | `0.1.44` | Transitive registry | MIT |
| `tracing-core` | `0.1.36` | Transitive registry | MIT |
| `typenum` | `1.20.1` | Transitive registry | MIT OR Apache-2.0 |
| `unicode-ident` | `1.0.27` | Transitive registry | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `untrusted` | `0.9.0` | Transitive registry | ISC |
| `ureq` | `3.5.2` | Direct registry | MIT OR Apache-2.0 |
| `ureq-proto` | `0.7.1` | Transitive registry | MIT OR Apache-2.0 |
| `url` | `2.5.9` | Transitive registry | MIT OR Apache-2.0 |
| `utf8-zero` | `0.8.1` | Transitive registry | MIT OR Apache-2.0 |
| `utf8_iter` | `1.1.0` | Transitive registry | Apache-2.0 OR MIT |
| `utf_types` | `1.0.0` | Transitive registry | Apache-2.0 OR MIT |
| `vcpkg` | `0.2.15` | Transitive registry | MIT/Apache-2.0 |
| `version_check` | `0.9.5` | Transitive registry | MIT/Apache-2.0 |
| `wasi` | `0.11.1+wasi-snapshot-preview1` | Transitive registry | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `webpki-roots` | `1.0.12` | Transitive registry | CDLA-Permissive-2.0 |
| `windows-link` | `0.2.1` | Transitive registry | MIT OR Apache-2.0 |
| `windows-sys` | `0.52.0` | Transitive registry | MIT OR Apache-2.0 |
| `windows-sys` | `0.61.2` | Transitive registry | MIT OR Apache-2.0 |
| `windows-targets` | `0.52.6` | Transitive registry | MIT OR Apache-2.0 |
| `windows_aarch64_gnullvm` | `0.52.6` | Transitive registry | MIT OR Apache-2.0 |
| `windows_aarch64_msvc` | `0.52.6` | Transitive registry | MIT OR Apache-2.0 |
| `windows_i686_gnu` | `0.52.6` | Transitive registry | MIT OR Apache-2.0 |
| `windows_i686_gnullvm` | `0.52.6` | Transitive registry | MIT OR Apache-2.0 |
| `windows_i686_msvc` | `0.52.6` | Transitive registry | MIT OR Apache-2.0 |
| `windows_x86_64_gnu` | `0.52.6` | Transitive registry | MIT OR Apache-2.0 |
| `windows_x86_64_gnullvm` | `0.52.6` | Transitive registry | MIT OR Apache-2.0 |
| `windows_x86_64_msvc` | `0.52.6` | Transitive registry | MIT OR Apache-2.0 |
| `writeable` | `0.6.4` | Transitive registry | Unicode-3.0 |
| `yoke` | `0.8.4` | Transitive registry | Unicode-3.0 |
| `yoke-derive` | `0.8.4` | Transitive registry | Unicode-3.0 |
| `zerofrom` | `0.1.8` | Transitive registry | Unicode-3.0 |
| `zerofrom-derive` | `0.1.8` | Transitive registry | Unicode-3.0 |
| `zeroize` | `1.9.2` | Transitive registry | Apache-2.0 OR MIT |
| `zerotrie` | `0.2.5` | Transitive registry | Unicode-3.0 |
| `zerovec` | `0.11.9` | Transitive registry | Unicode-3.0 |
| `zerovec-derive` | `0.11.7` | Transitive registry | Unicode-3.0 |
| `zlib-rs` | `0.6.9` | Transitive registry | Zlib |
| `zmij` | `1.0.23` | Transitive registry | MIT |

## Web entries

| Package | Locked version | Role | Declared license |
| --- | --- | --- | --- |
| `@esbuild/aix-ppc64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/android-arm` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/android-arm64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/android-x64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/darwin-arm64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/darwin-x64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/freebsd-arm64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/freebsd-x64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/linux-arm` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/linux-arm64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/linux-ia32` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/linux-loong64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/linux-mips64el` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/linux-ppc64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/linux-riscv64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/linux-s390x` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/linux-x64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/netbsd-arm64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/netbsd-x64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/openbsd-arm64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/openbsd-x64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/openharmony-arm64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/sunos-x64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/win32-arm64` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/win32-ia32` | `0.28.2` | Transitive development optional | MIT |
| `@esbuild/win32-x64` | `0.28.2` | Transitive development optional | MIT |
| `@rollup/rollup-android-arm-eabi` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-android-arm64` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-android-x64` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-darwin-arm64` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-darwin-x64` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-freebsd-arm64` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-freebsd-x64` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-arm-gnueabihf` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-arm-musleabihf` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-arm64-gnu` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-arm64-musl` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-loong64-gnu` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-loong64-musl` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-ppc64-gnu` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-ppc64-musl` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-riscv64-gnu` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-riscv64-musl` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-s390x-gnu` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-x64-gnu` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-linux-x64-musl` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-openbsd-x64` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-openharmony-arm64` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-win32-arm64-msvc` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-win32-ia32-msvc` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-win32-x64-gnu` | `4.68.8` | Transitive development optional | MIT |
| `@rollup/rollup-win32-x64-msvc` | `4.68.8` | Transitive development optional | MIT |
| `@types/estree` | `1.0.10` | Transitive development | MIT |
| `@types/react` | `19.3.1` | Direct development | MIT |
| `@types/react-dom` | `19.3.0` | Direct development | MIT |
| `csstype` | `3.2.4` | Transitive development | MIT |
| `esbuild` | `0.28.2` | Transitive development | MIT |
| `fdir` | `6.5.0` | Transitive development | MIT |
| `fsevents` | `2.3.3` | Transitive development optional | MIT |
| `nanoid` | `3.3.20` | Transitive development | MIT |
| `picocolors` | `1.1.1` | Transitive development | ISC |
| `picomatch` | `4.0.7` | Transitive development | MIT |
| `postcss` | `8.5.33` | Transitive development | MIT |
| `react` | `19.3.1` | Direct | MIT |
| `react-dom` | `19.3.1` | Direct | MIT |
| `rollup` | `4.68.8` | Transitive development | MIT |
| `scheduler` | `0.28.0` | Transitive | MIT |
| `source-map-js` | `1.2.4` | Transitive development | BSD-3-Clause |
| `tinyglobby` | `0.2.17` | Transitive development | MIT |
| `typescript` | `5.9.3` | Direct development | Apache-2.0 |
| `vite` | `7.3.8` | Direct development | MIT |
