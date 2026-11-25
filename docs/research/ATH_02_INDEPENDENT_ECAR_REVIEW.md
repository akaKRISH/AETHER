# ATH-02 independent ECAR research review

**BLOCKED.** Reviewed checkpoint `7bef4a921ea0e8bf49393617249a0e4114baed69`. This was a review of the frozen design and source. No held-out outcomes were inspected or generated. The [original specification](../../benchmarks/athena-ecar-heldout-spec.json) matched its recorded SHA-256 `876cbba77e9746dca4bab1212890e3c1bc57c19a82ab047e8fa9a5362fce1e00`. Preserve the reported pilot estimate of **+0.17 utility points**, with its reported 95% interval **−0.84 to +1.19**, as historical development output. The seed-key defect below limits the interval's interpretation.

| Review area | Finding |
|---|---|
| Matched budgets | All four arms receive the same 24, 48 or 84 simulated wall units, 48 attempts and quarter verification reserve for a given case. Exploration costs three units and task-specific replay and challenge prices apply equally. Reserve and settle precede outcome updates, so failed work is charged. These are matched simulated caps. Actual host scheduling time is measured separately. |
| Baseline fairness | The primary ECAR vs adaptive-flat comparison changes descendant weighting while sharing the candidate rules and charges. Fixed graph vs fixed flat isolates the same weighting change under fixed allocation. Fixed arms explore first and cap verification at half the wall budget, so adaptive-vs-fixed contrasts include both the order and access to verification spend. Report them as descriptive policy comparisons. |
| Seed independence | The draw uses `seed XOR stream` in the low bits before mixing. For a fixed family and claim, truth at seed 8193 and exploration failure at seed 8203 both enter the mixer with low word 8192 because `8193 XOR 1 = 8203 XOR 11`. The 30 frozen seeds and six streams that affect outcomes form 180 pairs with only 62 distinct pre-mix keys. This was verified from integer keys alone. Disjoint seed ranges do not prevent this within-split dependence. |
| Held-out leakage | The public JSON exposes seeds, scenario structure, truth probabilities and noise settings. The deterministic generator is in the same repository. Anyone with both can reconstruct the outcomes. The protocol records no independent custody of these inputs or labels. The public specification can support a disclosed synthetic replication, while a blinded confirmatory study requires independently controlled inputs. |
| Claim dependencies | Parents precede children and each claim has at most one parent. Latent truth requires true ancestors. Both treatments require observed verification of the parent before replay or challenge, so graph weighting affects priority under a common eligibility rule. The flat, branching, deep-chain, unreliable and noisy cases cover useful controls. The five designed single-parent families limit inference about general dependency graphs. |
| Statistics and uncertainty | The primary contrast pairs normalized utilities within family, cap and seed and averages those differences within seed. Resampling whole seed blocks correctly treats the 15 cases within each seed as dependent. The exact cross-seed key collisions violate the intended independent-block design. The 30-block floor and five-point planning effect are pilot sizing assumptions. They do not establish power for new families or an independently curated study. |
| Preregistered metrics | The protocol specifies the primary contrast, secondary descriptive contrasts, bootstrap resampling, coverage, false-verification rate, unresolved high-impact claims, replay, failures and cost. The summary rejects duplicate rows and usually fails on a missing arm within a present case. It learns the case set from the report, so a wholly missing seed, family or cap can pass silently. It also lacks a check against the registered 48-attempt cap and the frozen spec identity. |
| Threats to validity | Utility depends on designed weights, simulated verification error and fixed action costs. The same family/cap grid repeats across seeds. The resulting interval describes sampling variation conditional on the fixed synthetic design. Transfer to real tasks, different dependency structures or different verification systems remains untested. |

Evidence comes from the [diagnostic driver and draw function](../../crates/aether-engine/examples/ecar_diagnostics.rs), [scheduler](../../crates/aether-engine/src/scheduler.rs), [budget implementation](../../crates/aether-engine/src/budget.rs), [analysis specification](ATHENA_ECAR_PROTOCOL.md), and [summary script](../../scripts/athena_ecar_summary.py). The driver checks `phase != "development"` before running, so the frozen `heldout` input has no authorized execution path at this revision.

## Minimum corrections

1. Freeze a successor execution record that pins source revision, toolchain, runner, input digest and analysis. Give the held-out runner an explicit phase gate tied to the approved input hash. Keep the original JSON as an archival design. Use independently held inputs and labels if reporting a blinded confirmatory result. Obtain reviewer sign-off before opening outcomes.
2. Use an unambiguous encoding of split, seed, family, claim and stream and a reproducible pseudo-random draw from that encoding. Validate the complete key space before execution. Share each family's case draws across arms and caps within a seed. Use a distinct split identifier for development and held-out draws. Freeze this change before outcome generation and identify it as a protocol amendment.
3. Freeze the analysis against the approved case grid and report provenance. Reject missing, extra or duplicated cases before aggregation. Check the input and runner pins, arm set, both caps and metric denominators. Keep the paired seed-level contrast, 2,000 bootstrap draws using resampling seed 617 and percentile ranks 50 and 1950 unchanged. Keep secondary analyses descriptive. Document the metric denominators and the high-impact threshold of direct weight at least nine. Label the 30-block floor as a planning choice.

## Exact execution instructions after clearance

The current checkpoint is blocked. Complete the corrections and review them on development fixtures before the custodian releases the approved input to the executor. For the archived five-family, three-cap, 30-seed synthetic design the complete grid has 450 cases and 1,800 rows. For a separately curated grid derive the expected keys from its independently approved specification.

From the reviewed source revision, build the release example below. The successor execution record must pin the resulting executable and amended analysis script. The commands use the corrected runner's single input argument and the amended analysis script's two positional arguments, export then specification. Confirm the SHA-256 values of the executable, specification and script against that record before launching any held-out run. To avoid altering raw bytes, use PowerShell 7.4 or later and give the run a new output directory.

```powershell
cargo build --locked --release -p aether-engine --example ecar_diagnostics
if ($LASTEXITCODE) { throw 'Build failed' }
$runner = (Resolve-Path 'target/release/examples/ecar_diagnostics.exe').Path
$analysis = (Resolve-Path 'scripts/athena_ecar_summary.py').Path
$spec = (Resolve-Path $approvedSpecPath).Path
foreach ($pin in @(
    @{ Path = $runner; Hash = $approvedRunnerSha256 },
    @{ Path = $spec; Hash = $approvedSpecSha256 },
    @{ Path = $analysis; Hash = $approvedAnalysisSha256 }
)) {
    if ((Get-FileHash -Algorithm SHA256 -LiteralPath $pin.Path).Hash -ne $pin.Hash) {
        throw "Frozen pin mismatch for $($pin.Path)"
    }
}
$runDir = Join-Path 'artifacts' ('ath-02-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $runDir -ErrorAction Stop | Out-Null
$raw = Join-Path $runDir 'outcomes.json'
$summary = Join-Path $runDir 'summary.json'
& $runner $spec > $raw
if ($LASTEXITCODE) { throw 'Held-out runner failed' }
python $analysis $raw $spec > $summary
if ($LASTEXITCODE) { throw 'Held-out analysis failed' }
Get-FileHash -Algorithm SHA256 -LiteralPath $raw, $summary
```

Use the record's approved spec path and digests for the four PowerShell variables beginning with `$approved`. Retain the raw and summary hashes, the command log and the execution record together. The custodian reviews grid completeness and provenance before releasing only the predeclared statistics. If any check fails, preserve the failure record and return to review before retrying. Record the observed scope and uncertainty alongside the result.
