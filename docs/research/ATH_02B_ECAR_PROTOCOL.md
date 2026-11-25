# ATH-02B ECAR protocol amendment

Protocol ID `aether.ecar/2`. This amendment applies to future synthetic diagnostics. The archived protocol and pilot remain reproducible through the version 1 development path. The historical pilot estimate is **+0.17 utility points**, with its reported 95% interval **−0.84 to +1.19**. ATH-02 identified cross-seed dependence in that interval. [Original protocol](ATHENA_ECAR_PROTOCOL.md) · [Independent review](ATH_02_INDEPENDENT_ECAR_REVIEW.md)

## Methods

- Keep the primary hypothesis and estimand. Pair ECAR minus adaptive flat within family, cap and seed, in percentage points of true verified utility over total direct claim weight. Average family/cap differences within seed, then average seed blocks equally. Use sample SD and the percentile interval from 2,000 whole-block resamples with Python random seed 617 and sorted ranks 50 and 1950. The 30-block minimum and five-point planning effect are planning choices. Report fixed-arm contrasts and per-family estimates descriptively.
- Keep scenario eligibility, costs, truth dependencies, noisy verification, four arms, scheduler and weights. All arms share the case's simulated wall cap, 48 attempts and quarter verification reserve. The fixed arms additionally explore first and cap verification spending at half the wall cap.
- Version 2 replaces the XOR input with SHA-256 of the fixed prefix `aether.ecar.draw/2\0`, a split byte (0 development, 1 heldout), the 32-byte seed key, the seed as four big-endian bytes, and one byte each for family index, claim index and stream ID. Family index is the order in the committed scenario list. Interpret the first eight digest bytes as an unsigned big-endian integer and reduce modulo 100. Truth uses stream 1, and existing failure and noise use the original action markers 11, 21 and 31 and their successors. Arm and cap are absent from the key, so eligible observations share draws within each matched case. Distinct encoded keys can produce the same 0–99 value.
- Version 2 pins the executable, protocol, configuration and analysis. The analyzer reconstructs the full family × seed × cap × arm grid from the configuration and checks provenance, denominators and budget before aggregation. The proposed original dimensions yield 450 matched cases and 1,800 rows. Deduplicate family IDs and caps, and use at least 30 seed blocks for held-out execution.
- Count coverage as verified plus rejected divided by total claims. Count false verifications over all verified claims. Count high-impact unresolved claims using direct weight at least nine and excluding verified and rejected claims. Report replay passes and attempts, failed attempts, simulated wall charge per true verification, budget utilization and measured host/scheduler time. Use a missing ratio when its denominator is zero. The fixed synthetic families and single-parent forests bound any inference.

## Independent custody and release

An independent custodian controls the inputs for a new blinded study. The archived public held-out design can inform scenario construction. The custodian selects or approves the private inputs without reference to outcomes and generates a secret 32-byte key from an OS cryptographic random source outside the project workspace. Use configuration schema `aether.ecar-config/2` with `phase` set to `heldout`, a hex `seed_key`, `first_seed`, `seed_count`, `caps` and `scenarios`. The key supplies entropy even when block numbers and scenario templates are public. Private scenarios, if used, follow the same validated schema. Commit to the exact configuration bytes by recording their SHA-256 in the execution manifest and retain the bytes and key under custodian control.

Before any release, record the custodian, timestamp, study scope, source revision, Rust and Python versions, and manifest digest. The owner and independent reviewer approve the exact manifest digest through an authenticated channel. The manifest schema `aether.ecar-approval/1` contains `version`, `protocol_sha256`, `runner_sha256`, `config_sha256` and `analysis_sha256`. Hashes bind bytes. The independent approval record establishes authority. The operator supplies that digest separately from the manifest. The approving party must obtain the protocol, executable and analysis hashes from the reviewed build, and the configuration commitment directly from the custodian.

After the review and freeze, the custodian supplies the committed configuration only to the authorized executor. Use a trusted local operator and an immutable copy of the reviewed executable. The custodian retains the raw output and reports only preregistered summaries after checking grid completeness and provenance. Release the key or private inputs only after the preregistered analysis and disclosure decision. Preserve failure records and require review before retrying or changing any pin. This repository contains the custody procedure and public test fixtures. Independent appointment and private input release remain pending.

## Running the approved bundle

The Windows example is built with `cargo build --locked --release -p aether-engine --example ecar_diagnostics`. Use the pinned executable distributed from that build. With PowerShell 7.4 or later, run the commands in a new private output directory. Set the paths below to the reviewed bundle and supply `$approvedDigest` from the authenticated approval record. Keep the amended protocol file next to the analysis in the repository layout. The runner checks the approval digest, embedded protocol, executable and input bytes before execution. The analyzer checks the approved manifest, configuration, protocol and its own bytes, then compares the report's recorded pins before generating statistics.

```powershell
& $runner --heldout $config $manifest $approvedDigest > outcomes.json
if ($LASTEXITCODE) { throw 'Held-out runner rejected the input or failed' }
python $analysis --heldout outcomes.json $config $manifest $approvedDigest > summary.json
if ($LASTEXITCODE) { throw 'Held-out analysis rejected the report or failed' }
Get-FileHash -Algorithm SHA256 -LiteralPath outcomes.json, summary.json
```

For public development fixtures, use `--development-v2 <config>` on the runner and `--development-v2 <report> <config>` on the analyzer. Use at least two seed blocks for the sample SD. The original runner command with one argument accepts the exact [archived pilot configuration](../../benchmarks/athena-ecar-development.json). The one-argument analyzer accepts archival version 1 development reports.
