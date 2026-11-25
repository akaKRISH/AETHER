# Athena reproducibility materials

This manifest records a synthetic development replay, two local inference calibrations, loopback timings and a frozen sorting draft. Athena starts from `d3586bf` and records the combined Windows checkpoint at the local tag `athena-windows-local`. It accompanies the [working paper](ATHENA_WORKING_PAPER.md) and [diagnostic protocol](../../docs/research/ATHENA_ECAR_PROTOCOL.md).

## Frozen configurations

| File | SHA-256 | Status |
|---|---|---|
| [ECAR development configuration](../../benchmarks/athena-ecar-development.json) | `76c9b935fb52b3b84eddc4afdb350fbe103e479ccf37747b9f69196e2434675b` | 1,440 outcomes from five task families, three caps, 24 seed blocks and four policies |
| [Synthetic held-out specification](../../benchmarks/athena-ecar-heldout-spec.json) | `876cbba77e9746dca4bab1212890e3c1bc57c19a82ab047e8fa9a5362fce1e00` | Frozen and unexecuted. The development runner accepts only the development phase. |

The held-out specification is visible in the repository. Formal evaluation requires an independently controlled task and label set, a pinned analysis and policy revision, and independent reviewer approval.

## Replay the development analysis

From the repository root, verify the two configuration hashes above with `Get-FileHash -Algorithm SHA256`. With Rust and Python installed, use PowerShell 7.4 or later to run two fresh diagnostic processes and keep the new files under a separate artifact directory.

```powershell
$replay = Join-Path 'artifacts/athena' ('repro-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $replay -Force | Out-Null
foreach ($name in 'first.json', 'second.json') {
    cargo run --locked --release -p aether-engine --example ecar_diagnostics -- benchmarks/athena-ecar-development.json > (Join-Path $replay $name)
    if ($LASTEXITCODE -ne 0) { throw "Diagnostic run failed for $name" }
}
python scripts/athena_ecar_summary.py (Join-Path $replay 'first.json') > (Join-Path $replay 'summary.json')
if ($LASTEXITCODE -ne 0) { throw 'Summary generation failed' }
@'
import json
from pathlib import Path
import sys

def outcomes(path):
    return json.loads(Path(path).read_bytes())["outcomes"]

first, second, recorded = map(outcomes, sys.argv[1:])
assert len(first) == 1440
assert first == second == recorded
print("All 1,440 outcome rows agree")
'@ | python - (Join-Path $replay 'first.json') (Join-Path $replay 'second.json') research/results/athena-ecar-pilot.json
```

Outcome rows should agree exactly. Host wall time and measured scheduler time can vary between processes. The retained replay at `artifacts/athena/ecar-repeat.json` matched every recorded outcome row. Running the summary script on the recorded first run reproduced the checked-in summary as JSON, including the timing values copied from that run. These checks concern the synthetic diagnostic and its analysis. [Diagnostic runner](../../crates/aether-engine/examples/ecar_diagnostics.rs) · [Summary script](../../scripts/athena_ecar_summary.py)

## Approved local campaign

On 25 November 2026, the owner approved `athena-local-01` at the exact digest below. The preflight matched the stored protocol and the objective task pack, selected registered insertion sort, and confirmed the original executable bytes before approval. The frozen plan uses seed 19 and 96 generated signed integers, requires replay, and caps execution at five attempts and 12,000 ms. The existing draft preserves the model provenance, inference settings and Crossref citation.

| Pin | Exact value |
|---|---|
| Campaign ID | `athena-local-01` |
| Approved protocol SHA-256 | `3906c69297b1d1582c6f85cb35f44876e5f0d38c7c588e6ea877bfa31bb53dc9` |
| Evaluator executable SHA-256 | `e79057c75e9a04ec9a48a951c0662669d609518fa52b9d57046ff9b0f044daa2` |
| Policy SHA-256 | `a5d303db49b5b2ee4b8c1537a0df74705990fb4cc25c52c0e67809a888ce9754` |
| Ollama model manifest SHA-256 | `e0b8c85c75aad2573ff49bec177ebb1013772fba665a53d64c04266439a2222d` |

The run completed three of five attempts: original input, independent fresh-process replay and distinct challenge. Each attempt succeeded, and each trusted exact-reference fraction was 1.0. Replay retained the original input digest. Challenge input used seed 2236246020 and a different digest. The pinned evaluator verified the frozen claim. The proposed follow-up claim remains unresolved. The campaign used 140 ms of its 12,000 ms wall budget.

The complete report file SHA-256 is `7b1d9d9de5784b830d864cee0930ecc8f30c6dcc2cb28c5c697d4cba775e6ad2`. A second export from the same database and pinned executable matched the saved report byte for byte. [Campaign report](../results/athena-local-campaign.json) · [Approval and continuation record](../../RESUME.md)

## Recorded artifacts

| Artifact | SHA-256 | Contents |
|---|---|---|
| [Development output](../results/athena-ecar-pilot.json) | `dd09dbe1a1f69f01770805c7ab9c3429bda34fcef32444291d4f7d66637df3c3` | 1,440 outcome rows and measured host timing |
| [Paired summary](../results/athena-ecar-summary.json) | `6c7026fdea2c48a6476f0297d969738ffe996422239c2e11c6cc0ad840a2e190` | Utility, observed verification errors, paired contrasts and uncertainty |
| [Sorting objective](../../taskpacks/athena-sorting-objective.json) | `91374ae9b9d4bf3fc21a91176698943bcebf54f954b29063af626eb33fa06aac` | Original inputs to the local planner |
| [Model draft output](../results/athena-local-draft.json) | `daa49390eaddc4f2d8aec0bfc2356c39870a92941ea7da0deff943640ac3a3e2` | Ranked methods, provider provenance and inference usage |
| [Frozen draft export](../results/athena-local-draft-report.json) | `022d822875389e7db8b6dda151e96729e83e873d7496140133263d2d2967c04d` | Frozen protocol, cited metadata and creation event |
| [Approved campaign report](../results/athena-local-campaign.json) | `7b1d9d9de5784b830d864cee0930ecc8f30c6dcc2cb28c5c697d4cba775e6ad2` | Original, replay and challenge receipts, trusted verdict, three attempts and budget settlement |
| [Initial Ollama calibration](../results/athena-ollama-inference.json) | `ffc3132d95ae1e1dcc6bb2b506f49037bebc3e1d2f3a3bb3e07ba608db910cb1` | Ten requests with prompt, stream timing, model residency, RAM and device samples |
| [Ollama context calibration](../results/athena-ollama-contexts.json) | `212854b509f22ff9d74a63d0166c39b0389bd11e55b8ab49912fcdcc1a7b7e66` | Ten requests across the requested 2,048- and 4,096-token contexts and raw resource samples |
| [Loopback baseline](../results/athena-loopback-before.json) | `eaeaadb8d24756c65e9abb05b55f1b96def9a81fe4c2a8b5448ecfe597e53d18` | Windows release binary digest, four route distributions and peak service working set |
| [Loopback profile after the change](../results/athena-loopback-after.json) | `1ab48c458d3c2a293d7e4f8325c4f7f960954e3cfc2a0acfa9904fa1d4698713` | Same route workloads on the rebuilt Windows release, including the new report path |
| [Final release loopback recheck](../results/athena-loopback-recheck.json) | `5730e098dde42fdb215499842871b53a9415f9444494d670029edd9c11d848e2` | Repeated the same route workload on the final binary |

The inference captures describe the installed model and one Windows host. The loopback harness uses a temporary fixture campaign and sequential localhost reads. These measurements have different timing boundaries from the ECAR diagnostic. [Inference qualification](../../docs/operations/ATHENA_INFERENCE.md) · [Inference harness](../../scripts/athena_ollama.py) · [Loopback harness](../../tests/measure-loopback.cjs)

To repeat the loopback workload, run `node tests/measure-loopback.cjs` against a fresh release build. Pass the path to the retained evaluator as the first argument to measure the pre-optimization executable. Each output records the binary digest, sample counts and raw observations. Run both on an otherwise idle machine for a direct comparison.
