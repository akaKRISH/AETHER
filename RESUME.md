# Resume AETHER

## Checkpoint and phase

The incoming baseline was `d3586bf` (`prometheus-windows-verified`) on `main`. Recovery commit `41d3919` and foundation `c017c05` remain in history. The current validated Windows checkpoint has local tag `athena-windows-local`.

## Preserved implementation

- Frozen sorting and maximum-subarray pipelines, reviewed worker admission, journal snapshots, replay, and evidence export live in `crates/aether-api`, `crates/aether-runner`, and `crates/aether-store`.
- Model and literature adapters are in `crates/aether-api/src/planning` and `crates/aether-api/src/literature`. Four bounded knapsack arms and Criterion harnesses are in `crates/aether-engine` and `crates/aether-store`.
- The new browser controls, test harnesses, research dumps and delivery docs are in `web`, `tests`, `research/results`, and `docs`. The original sorting report SHA-256 remained `55ef0a797a07c7ce2e01538c3dd67e7e030621e14cd8b2bb19949cd8e1159b08`.

## Latest verification

Baseline passed 33 Rust and three frontend tests. The final Windows run passed 50 Rust and three frontend tests, format, strict Clippy, release build, frontend lint/type/build, browser smoke, advisory checks, and evidence validation. The release re-exported three prior reports byte for byte and completed a new objective-derived campaign with advice frozen under protocol v3. The hardware CLI observed the physical NVIDIA device. Crossref, OpenAlex and arXiv public probes returned 200. The recovery drill reconciled one interrupted lease. [Tests](docs/testing/TEST_RESULTS.md) · [Measurements](docs/research/BENCHMARK_RESULTS.md)

Athena passed 53 Rust tests and three frontend tests, Windows release and Chromium desktop/mobile checks. The four prior campaign exports and the new draft matched byte for byte. The Job Object descendant test and separate interrupted-lease release drill passed. Two fresh diagnostic processes matched all 1,440 outcome rows. [Test results](docs/testing/TEST_RESULTS.md#athena-windows-checks) · [Reproducibility](research/publications/ATHENA_REPRODUCIBILITY.md)

## Exact next action

Await explicit approval for `athena-local-01` at SHA-256 `3906c69297b1d1582c6f85cb35f44876e5f0d38c7c588e6ea877bfa31bb53dc9`. Use the retained exact binary at `artifacts/athena/aether-lab-e79057c75e9a04ec9a48a951c0662669d609518fa52b9d57046ff9b0f044daa2.exe` for approval and execution. The current state is draft. [Frozen report](research/results/athena-local-draft-report.json)

After the owner approves this hash, verify the frozen file and executable against the pins. Run from the repository root and keep the original draft evidence. The latest release binary has different bytes and the newly tested Job Object implementation.

```powershell
$env:AETHER_DATA_DIR = (Resolve-Path '.aether/lab').Path
$protocol = '3906c69297b1d1582c6f85cb35f44876e5f0d38c7c588e6ea877bfa31bb53dc9'
$evaluator = 'e79057c75e9a04ec9a48a951c0662669d609518fa52b9d57046ff9b0f044daa2'
$binary = (Resolve-Path "artifacts/athena/aether-lab-$evaluator.exe").Path
if ((Get-FileHash "$env:AETHER_DATA_DIR/protocols/$protocol.json").Hash -ne $protocol -or
    (Get-FileHash $binary).Hash -ne $evaluator) { throw 'Frozen campaign pin mismatch' }
& $binary approve athena-local-01 $protocol
if ($LASTEXITCODE) { throw 'Approval failed' }
& $binary run athena-local-01
if ($LASTEXITCODE) { throw 'Execution failed' }
& $binary report athena-local-01 > research/results/athena-local-campaign.json
if ($LASTEXITCODE) { throw 'Report export failed' }
```

Review exact reference agreement and the new replay and challenge attempts. Compare a second report export against the saved bytes and record the owner decision with the observed result.

## Remaining dependencies

The desktop Ollama process still reports cloud enabled. An owned process with the existing local model verified cloud disablement and completed calibration on the RTX 3050 at two confirmed context lengths. [Raw measurements](research/results/athena-ollama-contexts.json)

The approved-task Windows profile has process lifecycle containment. OS-enforced CPU and memory ceilings, filesystem and network boundaries, and restricted identities require separate qualification. Formal held-out testing requires independent review and task custody. See [next actions](NEXT_ACTIONS.md).
