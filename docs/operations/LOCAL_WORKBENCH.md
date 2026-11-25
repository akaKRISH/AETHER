# Run the local workbench

The native binaries require Rust 1.96.1 on the verified Windows x64 MSVC toolchain. The interface was developed with Node 24 and uses the committed npm lockfile. Commands below assume the repository root.

## Native fixture

```powershell
cargo build --locked --release --workspace
cargo run --locked --bin aether -- init
cargo run --locked --bin aether -- run-fixture
```

Copy the run ID into `aether verify <run-id>`. Copy the campaign ID into `aether report <campaign-id>`. Run IDs identify immutable native result records. [Native CLI storage](../../crates/aether-cli/README.md)

## Research campaign

```powershell
cargo run --locked --release --bin aether-lab -- demo sorting-demo 19
cargo run --locked --release --bin aether-lab -- report sorting-demo
```

To choose your own bounded inputs, copy the [plan example](../../taskpacks/example-campaign.json), set a fresh ID, and run `aether-lab plan <plan.json>`. Use the returned digest with `aether-lab approve <id> <sha256>`, then run `aether-lab run <id>`. Keep the same build through plan and run because each campaign pins the executable.

## Interface

Set a process-local token and start the workbench. The token is needed to enter the interface. Use a private terminal and keep the token out of shared logs.

```powershell
$env:AETHER_TOKEN = [Convert]::ToHexString([System.Security.Cryptography.RandomNumberGenerator]::GetBytes(32))
$env:AETHER_TOKEN
.\scripts\start-local.ps1
```

Open `http://127.0.0.1:8080`. The startup script installs pinned web dependencies, builds the UI and starts the loopback API. On Linux with the toolchain installed, run `sh scripts/start-local.sh` with a local token set. The Linux script is provided as a portable entry point. See the validation record for the platforms exercised. [Test results](../testing/TEST_RESULTS.md)

`AETHER_DATA_DIR` selects the private local state directory. The default is `.aether/lab`. Keep it on a local filesystem that supports hard links. For a consistent backup, stop all AETHER writers and copy `laboratory.sqlite3`, `literature.sqlite3` and `protocols/` to a private location. To check a restore, point the process-local `AETHER_DATA_DIR` at the restored copy and run `aether-lab report <id>` with its pinned executable. External transfers and public deployment require separate approval. [Restore drill](../testing/TEST_RESULTS.md)

Use `aether-lab recover <id>` after an interrupted supervisor has exited and its leases have expired. Recovery closes the campaign as failed and preserves its evidence. Use a fresh ID to run another campaign.

`aether-lab benchmark` runs the matched A–E simulation. `aether-lab benchmark-native` records measured process-inclusive timings for the reviewed sorting variants. Both are development checks. [Benchmark interpretation](../research/BENCHMARK_RESULTS.md)
