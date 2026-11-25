# Resume AETHER

## Checkpoint and phase

The incoming baseline was `d3586bf` (`prometheus-windows-verified`) on `main`. Recovery commit `41d3919` and foundation `c017c05` remain in history. The original Windows baseline is tagged `athena-windows-local`; the latest accepted checkpoint is `athena-engineering-verified`.

## Preserved implementation

- Frozen sorting and maximum-subarray pipelines, reviewed worker admission, journal snapshots, replay, and evidence export live in `crates/aether-api`, `crates/aether-runner`, and `crates/aether-store`.
- Model and literature adapters are in `crates/aether-api/src/planning` and `crates/aether-api/src/literature`. Four bounded knapsack arms and Criterion harnesses are in `crates/aether-engine` and `crates/aether-store`.
- The new browser controls, test harnesses, research dumps and delivery docs are in `web`, `tests`, `research/results`, and `docs`. The original sorting report SHA-256 remained `55ef0a797a07c7ce2e01538c3dd67e7e030621e14cd8b2bb19949cd8e1159b08`.

## DAE-01 constrained discovery

Run `cargo run --locked -p aether-api --bin aether-lab -- discover 19 96`. `discover` runs one seeded search over the reviewed 12-item knapsack fixture. It uses a checked 12-bit selection, one-bit mutations, eight distinct feasible parents ranked by value and weight, and periodic random proposals. Each proposal costs one of 8–96 slots, including duplicate and infeasible proposals. The engine reserves one slot to independently re-score the incumbent. It stops at the cap or after 24 consecutive proposals without an improved incumbent. Ties prefer lower weight, then the smaller mask. Exhaustive enumeration supplies the optimum for grading and has a fixed 12-item bound outside the proposal and check slots.

The JSON response contains the fixture, proposal lineage, termination reason, verification observation, regret and the SHA-256 of the report bytes. The local SQLite evidence store retains those bytes, retrievable through authenticated `GET /v1/artifacts/{sha256}`. The independent scoring path runs in the same process. The report is exploratory evidence and does not create a campaign or trusted claim verdict. To practice, trace a parent and child by slot, XOR their masks and check that exactly one bit changed.

Windows package tests passed 42/42 across engine, store and API. This includes the discovery budget, stagnation, lineage, repeatability, fresh CLI process and evidence integrity checks. Targeted strict Clippy, workspace formatting and documentation validation passed. The legacy `evolve 19 48` JSON matched the pre-change output. Frozen artifact SHA-256 checks matched, and the held-out study remains deferred. See the [execution queue](EXECUTION_QUEUE.md) for the next gate.

## DAE-02/03 paired development benchmark

Build with `cargo build --locked --release -p aether-engine --example discovery_benchmark`. Run `target/release/examples/discovery_benchmark run <new-report.json>`, then run `target/release/examples/discovery_benchmark verify <new-report.json>` as a separate process. The saved [raw export](research/results/dae-knapsack-development.json) replayed with evidence SHA-256 `2061ac45dff9e4d9ef240ca07b8e3a733497d08e5a6deff84e7bdf47d1e697db`. Full export SHA-256 is `94c955c6774b5fc218883fad3eb701987ba63e341844afdff3153f898c88fc5d`.

The grid paired three seeds, item counts 8, 12 and 16, two weight/value and capacity profiles, and budgets of 8 and 24 candidate evaluations. Each arm scored the empty baseline, charged repeats and infeasible proposals, and received one separate incumbent check. Each instance had one exhaustive oracle run after all six searches. Uniform random and evolutionary immigration share the same mask draw by evaluation slot, including across caps. Item draws, parent selection, mutation and greedy start use distinct domains. Greedy uses density order and cycles through seed-offset start items. ECAR remains a separate scheduling question because this grid holds verification fixed.

| Arm | Exact objective / 36 | Mean value gap | Median best value | Feasible evaluations / 576 | Mean unique masks | Median evaluations to best value | Median search time |
|---|---:|---:|---:|---:|---:|---:|---:|
| Random | 0 | 85.92 | 136 | 188 | 15.94 | 3.5 | 13.10 µs |
| Greedy | 23 | 1.42 | 173 | 576 | 6.19 | 2 | 11.05 µs |
| Evolution | 1 | 60.83 | 141 | 366 | 15.00 | 8 | 11.60 µs |

The optimized Windows run recorded 1,728 evaluations, 108 passing checks (8.2 µs total), 18 oracles (1.224 ms total), 5.707 ms evidence encoding and 3.327 ms serialization and buffered file flush. Search timing includes proposal construction, scoring, budget and lineage bookkeeping. Runtime is a single fixed-order host observation, and the two budgets share outcome prefixes. The fresh verifier checks the exact grid, all deterministic outcomes and lineage, content hash and timing shape. Elapsed timings are host observations. Greedy uses knowledge of the task structure, so this result describes the selected synthetic grid.

Fourteen engine and benchmark tests and the existing CLI persistence regression passed. Strict engine Clippy and formatting passed. The legacy `evolve 19 48` report matched its recorded digest and the frozen ECAR study remains deferred. [Execution queue](EXECUTION_QUEUE.md)

## DAE-04 adaptive hybrid discovery

The DAE-02/03 archive shows where evolution spent its budget. It evaluated 210 infeasible masks out of 576, including 129 of 216 initial or immigrant proposals. Greedy evaluated 576 feasible masks and reached its final best value at slot two in 21 of 36 trials, compared with two for evolution. Evolution retained more distinct masks. This diagnosed a useful tradeoff between directed construction and diversity on this grid, and motivated the ablations below.

Build the release `discovery_benchmark` example and run `hybrid-run <new-report.json>`, then `hybrid-verify <new-report.json>` in a fresh process. The [raw export](research/results/dae-hybrid-development.json) has evidence SHA-256 `62cfbfa4c0a80911cd3d5d1b9a698bcdd122a0dcd5dc2e65d5107cd910e4584a` and file SHA-256 `bc3654d2eb279884cca74f9a5d7f4fafde60e10930bc6cd06338eec256736ced`. The original report verified with its recorded evidence digest. Its baseline records are identical in the hybrid export, excluding timing observations.

The experiment uses the original 18 fixtures plus a density-friendly case and a density trap, with budgets of 8 and 24 scored candidates. Initialization consumes a slot. Variants seed with density greedy, constrain flips using remaining capacity, repair by removing low-density items, swap selected and unselected items, or generate a local neighbor by dropping one incumbent item and refilling without it. Fixed hybrid cycles through feasible flip, swap, and local refill. Adaptive hybrid uses the same operators and repairs. It samples unused or underused operators periodically and otherwise compares observed incumbent value gains per use. The oracle runs once per instance after all paired searches and checks.

| Arm | Optimal / 40 | Mean value gap | Feasible / 640 | Mean unique masks | Median search µs |
|---|---:|---:|---:|---:|---:|
| Random | 0 | 78.92 | 195 | 15.88 | 13.40 |
| Greedy | 25 | 1.48 | 640 | 6.22 | 11.45 |
| Evolution | 2 | 55.33 | 398 | 14.75 | 9.95 |
| Seeded | 21 | 3.27 | 392 | 14.38 | 14.45 |
| Seeded feasible flip | 20 | 3.48 | 523 | 13.32 | 10.30 |
| Seeded repair | 23 | 3.10 | 640 | 11.43 | 14.30 |
| Seeded swap | 20 | 2.83 | 427 | 14.53 | 12.45 |
| Seeded local | 28 | 1.35 | 523 | 10.97 | 11.90 |
| Fixed hybrid | 26 | 2.17 | 640 | 13.47 | 11.40 |
| Adaptive hybrid | 28 | 1.82 | 640 | 13.20 | 10.55 |

Seeded improved, tied or worsened against evolution in 32/8/0 pairs. Against seeded, feasible flip was 2/36/2, repair 4/35/1, swap 4/34/2, and local 15/24/1. Adaptive was 3/37/0 against fixed and 7/24/9 against greedy. Greedy solved the friendly case and missed the trap optimum by four points at both caps. Seeded local and both hybrids solved the trap at both caps. Feasible flips improved feasibility but slightly increased mean gap versus seeded. Repair made all scored candidates feasible, with fewer unique masks.

Search timing includes initialization, generation, scoring, budget and lineage. Traces count 12,012 density comparisons, 3,248 seed item visits, 1,387 constraint probes, 18,790 repair visits with 1,175 removals across 589 candidates, 8,422 local visits and 800 adaptive comparisons. Raw overweight masks and weights remain attached to their repaired candidates. The run scored 6,400 candidates, performed 400 passing incumbent checks (28.9 µs total) and ran 20 oracles (0.884 ms total). Evidence encoding took 30.398 ms and serialization with buffered flush took 11.245 ms. Candidate evaluations were matched. Operator work was counted separately and elapsed cost was measured in one Windows run with rotated arm order. The two caps share prefixes and the constructed trap demonstrates a specific mechanism, so the results remain exploratory.

The acceptance rerun from `05aac30` passed 18 targeted tests, strict engine Clippy, formatting, the release example build, documentation validation and Git diff checks. Fresh release processes verified the saved hybrid and baseline exports. Both file hashes and the frozen Athena and ECAR pins matched. The original timing observations were retained. The ECAR holdout remains deferred. [Execution queue](EXECUTION_QUEUE.md)

## Latest verification

Baseline passed 33 Rust and three frontend tests. The final Windows run passed 50 Rust and three frontend tests, format, strict Clippy, release build, frontend lint/type/build, browser smoke, advisory checks, and evidence validation. The release re-exported three prior reports byte for byte and completed a new objective-derived campaign with advice frozen under protocol v3. The hardware CLI observed the physical NVIDIA device. Crossref, OpenAlex and arXiv public probes returned 200. The recovery drill reconciled one interrupted lease. [Tests](docs/testing/TEST_RESULTS.md) · [Measurements](docs/research/BENCHMARK_RESULTS.md)

Athena passed 53 Rust tests and three frontend tests, Windows release and Chromium desktop/mobile checks. The four prior campaign exports and the new draft matched byte for byte. The Job Object descendant test and separate interrupted-lease release drill passed. Two fresh diagnostic processes matched all 1,440 outcome rows. [Test results](docs/testing/TEST_RESULTS.md#athena-windows-checks) · [Reproducibility](research/publications/ATHENA_REPRODUCIBILITY.md)

## Athena campaign result

On 25 November 2026, the owner approved protocol SHA-256 `3906c69297b1d1582c6f85cb35f44876e5f0d38c7c588e6ea877bfa31bb53dc9`. Before approval, the pinned protocol matched the objective task pack, chose the registered insertion sorter, used seed 19 and 96 items, and preserved five attempts and a 12,000 ms cap. The pinned executable SHA-256 remained `e79057c75e9a04ec9a48a951c0662669d609518fa52b9d57046ff9b0f044daa2`.

The approved run completed three successful attempts within its budget: exploration, fresh-process replay and challenge. Trusted exact-reference fractions were 1.0 on both 96-item seeds, 19 and 2236246020. The replay matched the exploration input digest, while the challenge input digest differed. The frozen claim was verified by the pinned evaluator and protocol rule. The additional follow-up claim remains proposed.

The report is `research/results/athena-local-campaign.json`, SHA-256 `7b1d9d9de5784b830d864cee0930ecc8f30c6dcc2cb28c5c697d4cba775e6ad2`. A separate report export matched it byte for byte. The attempt budget recorded three of five and 140 of 12,000 ms. Model inference happened during planning and used its separately recorded provider counters.

## ATH-02B research remediation checkpoint

ATH-02B implements the corrected version 2 synthetic protocol, SHA-256 `7e7f31346feb5b004a9f261d8c590d4e2c421e3a69fc023fe259353b0a1cc756`. It separates draw keys while sharing draws across matched arms and caps. Held-out execution requires an approved manifest digest that binds the protocol, executable, config and analysis. The analyzer rejects incomplete, duplicate, unexpected and unmatched cases before aggregation. The original protocol and archived pilot retain their bytes. The historical estimate remains +0.17 utility points with its reported 95% interval of −0.84 to +1.19. [Amendment and execution instructions](docs/research/ATH_02B_ECAR_PROTOCOL.md) · [Original review](docs/research/ATH_02_INDEPENDENT_ECAR_REVIEW.md)

Validation passed 11 Rust tests, 13 CLI regression tests against the release example, strict Clippy, formatting and documentation checks. The archival route reproduced all 1,440 development rows and the saved statistical summary.

Actual held-out execution remains pending. Appoint an independent custodian, commit the private version 2 input, and obtain owner and reviewer approval of the manifest digest before any release. Validation for this remediation used development and public synthetic fixtures.

## ATH-04 Windows worker hardening

The Windows Job Object now limits each reviewed worker and its job to one second of user-mode CPU, 64 MiB of aggregate committed memory, and one active process. Admission waits for the limits and attachment to succeed. The runner starts only the running platform executable with a typed built-in task, clears inherited environment variables and reaps on cancellation or wall timeout. [Controls and limits](docs/security/ATHENA_WORKER_CONTAINMENT.md)

Targeted Windows validation passed 14 runner and 22 API tests. The seven Windows protection tests also passed in the optimized build. Fixtures verified CPU quota termination, rejected large allocation and child launch, termination on job close, timeout and live cancellation, and a synthetic secret absent from the worker environment. Formatting and strict runner Clippy passed. The frozen Athena evaluator, campaign protocol and ECAR specification retained their recorded SHA-256 values.

## Remaining dependencies

The desktop Ollama process still reports cloud enabled. An owned process with the existing local model verified cloud disablement and completed calibration on the RTX 3050 at two confirmed context lengths. [Raw measurements](research/results/athena-ollama-contexts.json)

The reviewed Windows profile enforces resource and process limits under the desktop identity. A qualified backend for OS filesystem denial, network denial and restricted identity remains open. Formal held-out testing still requires independent review and task custody. See [next actions](NEXT_ACTIONS.md).

## ATH-05 final Athena integration check

Starting from `e0bcc3a`, `cargo fmt --all -- --check`, strict workspace Clippy, locked workspace tests and the release build passed. The suite reported 64 passed and 9 ignored; the optimized Windows containment selection reported 10 passed and 9 ignored. Frontend lint, TypeScript check, all three tests and production build passed. The release service started with a temporary token and data directory; `/health` returned `{"status":"ok","scope":"loopback"}` and `/` served HTML.

Two fresh development ECAR runs matched all 1,440 rows in the archived pilot. Frozen configuration, specification, pilot, summary, objective, drafts, campaign, calibration and loopback records matched their recorded SHA-256 values. The original pinned executable and version 2 protocol also matched. `node tests/browser-smoke.cjs` could not start because Node could not resolve `playwright`; browser UI coverage is therefore unverified in this checkpoint. The frozen development inputs, archived pilot and `docs/archive` have no changes since `athena-windows-local`. No held-out run was made.

### Athena release summary

Implemented capabilities include deterministic synthetic ECAR replay and matched analysis, a model-guided local sorting proposal with approved replay and challenge, and a reviewed Windows worker profile. Recorded measurements remain scoped to this host: warm local inference median 224–232 ms, sampled peak device-wide VRAM 3,775 MiB; the approved 96-item campaign used 140/12,000 ms and verified all three attempts. The synthetic ECAR development estimate remains +0.17 utility points (95% interval −0.84 to +1.19). Job Objects enforce one-second user-mode CPU, 64 MiB aggregate committed memory, one process and kill-on-close; the worker still runs as the desktop user without OS filesystem or network denial. Independent held-out custody and approval remain outstanding. The missing Playwright module blocks the browser gate, so `athena-engineering-verified` was not tagged.
At `e0396b5`, the documentation validator exited 1 because `jsonschema[format]` is unavailable and its negative timestamp case was accepted. It otherwise scanned 57 JSON files, 267 Markdown files, 1,245 relative destinations, 21 SVG views and 41 mapped requirements. No dependency was downloaded; this gate remained open alongside browser smoke.

## ATH-05B final acceptance repair

Starting from `e0396b5`, pinned `playwright@1.65.1` in the web package and added the browser smoke script. Its expected Chromium revision 1255 was already installed in `E:\AI_Models\ms-playwright`, so no browser download was needed. Desktop and 390-pixel mobile smoke passed, covering authentication, sorting and optimization campaigns, evidence, reports, planning, metadata gates, benchmark, GPU panel and responsive layout.

The timestamp false acceptance came from using `FormatChecker` without jsonschema's format extras; no `date-time` checker was registered. `requirements-validation.txt` now locks `jsonschema[format]` and its transitive dependencies. `tests/test_doc_formats.py` verifies a registered checker, valid timestamp acceptance and malformed timestamp rejection. All three tests passed, and the full documentation validator passed with 13 schemas, 57 JSON files, 268 Markdown files, 1,250 relative destinations and 5 negative schema cases.

`npm ci`, frontend lint, type-check, three frontend tests, build and browser smoke passed. Unchanged Rust gates and measurements from ATH-05 were reused. Frozen hashes for the pinned executable, both ECAR configs, archived pilot and summary, Athena drafts and report, measurement records and version 2 protocol still match. No held-out study ran. The worker remains limited to reviewed built-in tasks under Job Object resource limits; filesystem, network and restricted-identity isolation are not provided. All ATH-05B acceptance gates passed; the local checkpoint is tagged `athena-engineering-verified`.
