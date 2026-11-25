# Windows reviewed worker containment

The Windows supervisor creates an anonymous Job Object with kill-on-close before it launches a reviewed worker. The trusted worker entry point waits on a one-byte admission pipe before it reads the request. The supervisor attaches the process to the job before sending admission and refuses to run if attachment fails. Descendants inherit job membership under the default job policy, and the supervisor never enables breakaway. Closing the last job handle terminates members. [Windows Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) · [Runner source](../../crates/aether-runner/src/windows_worker.rs)

The supervisor retains existing wall-clock enforcement, stdout and stderr limits, the environment allowlist and typed receipt validation. It closes the job before joining pipe readers so a remaining descendant cannot maintain those pipes indefinitely. The controller charges interrupted attempts through its existing lease rules. [Runner](../../crates/aether-runner/src/lib.rs) · [Controller](../../crates/aether-api/src/lab.rs)

The supervisor uses the pinned, Windows-only `win32job` wrapper to avoid first-party unsafe handle code. Killing the directly observed child alone leaves descendants unmanaged. The attached job covers that lifecycle gap with one handle and no new runtime. Keep file, network and resource enforcement separate when qualifying a future restricted profile. [Dependency review](../governance/DEPENDENCY_INVENTORY.md#athena-lock-review)

| Control | Verified or outstanding |
|---|---|
| Process-tree termination | A harmless fixture led a Windows worker to spawn a descendant that wrote a ready marker. After the job closed, the parent was reaped and the descendant could never write its follow-up marker during the test window. |
| Cancellation and invalid output | Reviewed cancellation and bounded typed response handling remain part of the runner and its test coverage. The browser smoke uses its separate temporary state. |
| Workspace path and HTTP access | Identifier, asset path and bearer token checks protect AETHER's routes and storage operations. A Windows process still inherits the trusted account's access to local files. |
| CPU and memory | The supervisor tracks budget and enforces wall and stream limits. OS-enforced aggregate memory and CPU caps need qualification. |
| Filesystem, egress and identity | Job membership governs lifecycle. Separate execution credentials, filesystem sandboxing and network egress denial are outstanding. The worker receives a stripped environment, and that protects only the values passed in environment variables. |
| Portability | The qualified build is Windows x64. Linux containment and externally supplied programs require independent review and test evidence. |

All runnable algorithms come from the reviewed task registry. These lifecycle controls do not admit arbitrary programs or grant external code execution authority. [Threat model](THREAT_MODEL.md) · [Validation evidence](../testing/TEST_RESULTS.md)
