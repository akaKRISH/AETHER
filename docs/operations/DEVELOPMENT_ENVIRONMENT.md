# Development environment

Read-only host inventory on 24 November 2026.

| Probe | Observed |
|---|---|
| OS | Windows 11 Home Single Language, 64-bit, build 26300 |
| CPU | Intel Core i5-13420H, 8 physical cores, 12 logical processors |
| RAM and free project volume | 15.64 GiB reported by Windows; C: 54.43 GiB free at probe |
| GPU and driver | NVIDIA RTX 3050 6 GB Laptop GPU; 6144 MiB VRAM; driver 610.91 |
| Rust | rustup default stable-x86_64-pc-windows-msvc; rustc 1.96.1 and Cargo 1.96.1 |
| Other detected tools | Git 2.54.0.windows.1; Python 3.14.7; Node 24.20.0; Docker CLI 29.9.2 |
| WSL | Ubuntu and docker-desktop registered as WSL2, both stopped at probe |

The installed Rust release supports Edition 2024. The [SDK repair verification](#windows-sdk-repair-and-verification) records the native Windows toolchain check. WSL registration and a Docker CLI version establish availability signals. Worker isolation, GPU toolkit/runtime compatibility and the Docker daemon need separate qualification. The Phase 1 design fits the observed host. Application execution and capacity targets remain to be measured. [Rust 2024 release](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)

The source R0 digits task is future work. Confirm redistribution conditions for the precise packaged bytes and split, pin dependencies in a separate worker environment, and document its license and provenance. [scikit-learn load_digits](https://scikit-learn.org/stable/modules/generated/sklearn.datasets.load_digits.html)

No system settings or toolchain configuration changed during these checks.

## Phase 1 native build probe

On 24 November 2026, a dependency-free Cargo binary using Edition 2024 was written to a temporary directory. The active target was `x86_64-pc-windows-msvc`. Observed tools were Rust and Cargo 1.96.1, Git 2.54.0.windows.1, rustfmt 1.9.0 and Clippy 0.1.96.

| Check | Observed result |
|---|---|
| `cargo generate-lockfile --offline` and `cargo check --offline --locked` | Exit 0 in the temporary package. |
| `rustc --edition=2024 --emit=obj src/main.rs -o preflight.obj` | Exit 0 and emitted a 4,879-byte object file. |
| `cargo build --release --offline --locked` in default shell | Cargo exit 101. The shell could not locate `link.exe`. |
| Retry in x64 Visual Studio Build Tools developer environment | The installed MSVC 14.51.36231 linker was found. Its link attempt failed with `LNK1181` for `kernel32.lib`. The standard SDK library directory was absent. |
| Retry with Rust's bundled `rust-lld` | Cargo exit 101 with missing Windows import libraries, including `kernel32.lib`. |

The probe never produced a runnable executable. Native build qualification requires completing the Windows SDK installation or locating a complete x64 SDK and rerunning the probe in the developer environment. This preflight did not change system settings. [MSVC command-line environment](https://learn.microsoft.com/en-us/cpp/build/building-on-the-command-line)

## Windows SDK repair diagnosis

A Microsoft-signed portable `vswhere` 3.1.7 reported Visual Studio Build Tools 2026 version 18.9.2 (`18.9.12120.119`) at `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools`. The instance reported `isComplete=false`. The MSVC 14.51.36231 x64 linker and its `msvcrt.lib` were present. The usual Visual Studio Installer executable was absent.

The Windows Kits registry listed `10.0.26100.0`, and the uninstall inventory listed Windows SDK AddOn `10.1.0.0`. The registered kit root contained partial metadata and redistributables, with no `Include`, `Lib` or `bin` tree. No complete desktop SDK or x64 `kernel32.lib` was found in the registered kit, standard SDK paths, or Visual Studio package cache.

| Process-local x64 developer environment | Observed |
|---|---|
| `PATH` | Visual Studio's `bin\HostX64\x64` resolved `link.exe`. Cargo was present. Windows Kits entries were absent. |
| `LIB` | MSVC `lib\x64` only. |
| `INCLUDE` | MSVC and its auxiliary include paths only. |
| `WindowsSdkDir` / `WindowsSDKVersion` | Unset / lone backslash. |

In the process-local shell, a new dependency-free Rust 2024 package produced the following results.

| Command | Result before SDK installation |
|---|---|
| `cargo check` | Passed, exit 0. |
| `cargo build` | Failed, exit 101. MSVC `LNK1181` for `kernel32.lib`. |
| `cargo build --release` | Failed, exit 101. Same missing import library. |
| `cargo test` | Failed to link the test executable, exit 101. No unit test ran. |
| `cargo clippy -- -D warnings` | Passed, exit 0. |
| `cargo fmt -- --check` | Passed, exit 0. |
| Run release executable | Blocked. Build produced no executable, so the expected line `AETHER MSVC native probe OK` was unverified. |

The diagnosis identified the Windows 11 SDK 26100-series desktop x64 development files, shared headers, Windows import libraries and UCRT dependencies as the required addition. The corresponding Visual Studio component is `Microsoft.VisualStudio.Component.Windows11SDK.26100`. [Visual Studio Build Tools components](https://learn.microsoft.com/en-us/visualstudio/install/workload-component-id-vs-build-tools?view=vs-2026)

## Windows SDK repair and verification

On 25 November 2026, the owner authorized the SDK repair. The Microsoft-signed standalone Windows 11 SDK `10.0.26100.9457` installer ran with Windows elevation and selected `OptionId.DesktopCPPx64`. Microsoft Setup installed that feature and its bundled dependencies. The installer returned exit 0 and reported no restart required. Existing Rust and Cargo 1.96.1 and MSVC 14.51.36231 were reused. No global environment variables were edited by the probe. [Microsoft Windows SDK downloads](https://learn.microsoft.com/en-us/windows/apps/windows-sdk/downloads#windows-11--26100-versions)

The installed software entry reports `10.1.26100.9457`. The sole version directory found under `C:\Program Files (x86)\Windows Kits\10\Lib` was `10.0.26100.0`. The x64 `um\x64\kernel32.lib` and `ucrt\x64\ucrt.lib` are present. Microsoft `dumpbin` identified `kernel32.lib` as `8664 machine (x64)`. The matching Include tree contains `um\Windows.h`, `um\winsdkver.h`, `shared\winapifamily.h` and `ucrt\stdio.h`. A syntax-only MSVC compilation including `Windows.h` and `stdio.h` passed.

The dependency-free Rust 2024 probe ran outside AETHER, with `vcvars64.bat` setting the x64 linker, `WindowsSDKVersion=10.0.26100.0\`, both SDK library paths, and the SDK include paths.

| Command | Actual result |
|---|---|
| `cargo check` | Passed, exit 0. |
| `cargo build` | Passed, exit 0. |
| `cargo build --release` | Passed, exit 0. |
| `cargo test` | Passed, one test and zero failures. |
| `cargo clippy -- -D warnings` | Passed, exit 0. |
| `cargo fmt -- --check` | Passed, exit 0. |
| Release executable | Exit 0. Standard output was exactly `AETHER MSVC native probe OK` plus a newline. |

Fresh PowerShell 7.6.9 and Windows PowerShell 5.1.26100.9818 sessions were tested with `-NoProfile`, without a developer-shell initialization, and with independent build output directories. Each passed debug and release builds and ran its release executable with the expected output. **Native builds are unblocked on this host.** Plain PowerShell works with the tested MSVC target. If manual initialization is needed on another setup, run `vcvars64.bat` and Cargo in the same child shell to keep the settings process-local. [Microsoft command-line environment](https://learn.microsoft.com/en-us/cpp/build/building-on-the-command-line)

```powershell
$vcvars = Join-Path ([Environment]::GetFolderPath('ProgramFilesX86')) 'Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
cmd.exe /d /c "call `"$vcvars`" && cargo build --release"
```

These are host toolchain checks. Milestone 01 acceptance still requires an approved application implementation and its own tests.
