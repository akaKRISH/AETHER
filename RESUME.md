# Resume AETHER

## APO-06A adapter-training infrastructure and feasibility

Started from `ff48125`. The Rust controller publishes unapproved LoRA and QLoRA manifests with exact base and dataset identities, split provenance, adapter settings, resource caps, baseline identity, and regression requirements. The CLI, authenticated API and Intelligence Observatory expose proposals, feasibility and synthetic receipts. [Commands](docs/research/APOLLO_EXECUTION_PLAN.md#apo-06a-adapter-training-feasibility) · [Backend contract](docs/decisions/ADR-019.md)

The Windows Python checks covered the project and PATH Python 3.14.7 installations and the five registered interpreter entries. PyTorch, Transformers, PEFT, Accelerate, SafeTensors, bitsandbytes, TorchAO and Triton were absent in those checks. The RTX 3050 had 6,144 MiB of VRAM and 5,649 MiB free at the initial probe. NVIDIA reported driver 610.91 and CUDA UMD 13.3. The probe found no `nvcc` on PATH or toolkit under the standard installation directory. No PyTorch CUDA runtime was available to qualify. The workspace volume had 28,369,821,696 bytes free and the host had 1,206,865,920 bytes of available RAM at that probe.

The installed Qwen 3.5 4B Q4_K_M GGUF is an inference artifact for this backend. Its local header declares 32 blocks, embedding width 2,560 and 16 attention heads. Eight full-attention query tensors have shape 2,560 × 8,192. A GGUF content pin does not establish a trainable checkpoint. A single SafeTensors shard also leaves the config, tokenizer, complete tensor set and immutable runtime mount unqualified.

For a hypothetical four-step, rank-4 proposal on those eight query projections, the planner estimated 9.84 GiB RAM and 8.92 GiB VRAM for LoRA, and 4.47 GiB RAM and 3.54 GiB VRAM for QLoRA. Both estimated 2.54 GiB storage and 6.46 trillion dense-reference FLOPs. The QLoRA proposal observed 1.47 GiB available RAM. These estimates use the inferred geometry and public synthetic split controls. They provide no measurement of a trainable checkpoint or actual allocation. The saved Qwen manifests are `925eb5a9d56c72279f723e1ac1993913272d8505af6bd5ba290576b5a6f1d177` and `4adf9885a1184649d0a3e73542723f8581c3d977e77a091fda85061db408c2e7`.

Both synthetic manifests reproduced across fresh CLI processes. The independent Python fixture matched all four Rust receipts. Completion produced steps 2 and 4, and cancellation before step 3 preserved just step 2. Each recorded zero weight updates. The pinned APO-05 baseline still reproduced its recorded digest.

The affected API suite passed 39 unit and 18 platform tests. Coverage includes split overlap and mutated pins, isolated test custody declarations, resource arithmetic, synthetic receipt integrity, cancellation and authenticated read-only review. Strict Clippy, workspace formatting, the offline native release build, five frontend tests, lint, type checking, the frontend build, desktop and mobile browser smoke, and documentation and schema validation passed. The desktop browser opened the QLoRA proposal and its cancelled synthetic receipt. Frozen research artifacts, the model evaluator and the deferred Rune workspace match `ff48125`.

Real training remains deferred. The next gate needs a complete rights-reviewed native checkpoint, a pinned and reviewed Python worker and runtime, qualified resources and independent custody and evaluation. The approved model and ECAR evidence stay separate.

## APO-05 open-weight model research laboratory

Started from `77e135a`. The native registry retains inspected metadata, file identities and evidence-backed evaluation receipts. The CLI, authenticated API and Intelligence Observatory expose inspections, case results and matched regressions. [Commands](docs/research/APOLLO_EXECUTION_PLAN.md#apo-05-open-weight-model-laboratory) · [Design and restrictions](docs/decisions/ADR-018.md)

The local inventory read seven Ollama manifests. Four GGUF headers passed inspection. BGE-M3 had 389 tensors, Gemma 4 E4B had 2,131, Qwen 3.5 4B had 426 and Qwen 3.5 9B had 883. Three cloud aliases were excluded. The Qwen 3.5 4B configuration declares Q4_K_M, and its metadata declares Apache-2.0 with 248,320 tokenizer entries. Its inspected weight file is 2,648,593,216 bytes, describes 4,205,751,296 stored tensor elements and matched SHA-256 `a9d0a3e1d8c91732a3812becd31f8d4a8ff57414903f226306069f0437e2ac50`. The other models received header checks. SafeTensors validation used fixtures.

The scripted baseline scored 7/7 and reproduced byte-for-byte across fresh release CLI processes. The deliberately degraded fixture scored 5/7. Matched comparison identified the calibration and refusal regressions. Brier loss on five scored answers rose from 0.04 to 0.16. These are scripted controls for the evaluator. Local artifact digests are `acd399f1367b0106e6ddff50739939c373b4457ee66244d8d552d2455068b8c3` and `e4fa201ad70767032d098561129e11ae1ee6ab2ede721dbfca8ec394072be64f`.

Live admission stopped before any model call. After the build and browser checks, the release CLI observed 2,910,543,872 bytes of available RAM against a 5,471,658,963-byte policy estimate. The RTX 3050 reported 5,685 MiB free VRAM. Receipt `a85059e920b3e0160fb01ad081daf24992ceeabdc1341ee9123a5b690470a24d` contains the measured snapshots and zero calls. Live token, latency and peak-memory results remain unavailable for APO-05. Resume with the existing qualified cloud-disabled service and sufficient headroom.

Validation passed 36 API unit, 16 platform and 14 runner tests, including Windows containment checks. The final bounded parser and platform model workflow were rechecked. Strict Clippy, Rust formatting, offline native release build, five frontend tests, frontend lint and build, desktop and mobile browser smoke, and documentation and schema validation passed. The browser inspected a fixture tensor and opened model reports and a comparison. Historical artifacts, ECAR protocols and the deferred Rune workspace match `77e135a`.

## APO-04 automated scientific experimental design

Started from `b89d3b8`. The Rust CLI and authenticated Research OS generate three exploratory drafts from the pinned APO-03 graph. The plans cover negative results, profile boundaries and local improvement with repair. They record exact source revisions, confounders, paired controls, falsification rules and reproducibility manifests. Each review draft has a canonical SHA-256 identity and remains unapproved. [Commands](docs/research/APOLLO_EXECUTION_PLAN.md#apo-04-experimental-design) · [Design decision](docs/decisions/ADR-017.md)

The planned candidate counts are 320, 384 and 384. The matched grids reserve 100, 120 and 108 total attempts covering search, independent scoring and the isolated oracle. The respective aggregate wall ceilings are 70, 84 and 72 seconds. These are planning figures. No proposed experiment or ECAR held-out study ran.

Two fresh CLI processes produced byte-identical review JSON with SHA-256 `92d255a0d0343836ccf0e8ce8e42ca1a664d46a4faa05ede840f50a9d1a90765`. The validated protocol digests are `dd45aa84ef4c084353b8fa8ac0378758fb1a34f57e180087bf48295d754b489f` for negative results, `5ba8f71ed215007a7a998b37a96db09b075aafc16214b743ab54a4d34b551159` for the boundary, and `085974cb95640aee3b082406c2658bd64c3467af21359e961720b15728c6b112` for method composition.

Validation passed 31 Rust API unit tests, 15 platform tests, strict API Clippy, workspace formatting, the native API release build, five frontend tests, type checking, lint, production web build, desktop and mobile browser smoke, and documentation and schema checks. The CLI and HTTP campaign paths rejected the drafts. The approval gate also rejected a test record indexed with a digest matching the draft bytes. Frozen artifacts matched `b89d3b8`.

Next, qualify a fresh matched-case adapter and exact archive-input exclusion, register a reviewed paired knapsack worker and verifier, implement the exact joint factorial cell, and review the aggregate resource plan through the existing campaign workflow. Confirmatory work also needs independent held-out custody and justified sampling assumptions. Historical records remain frozen.

## APO-03 evidence-grounded hypothesis discovery

Started from `bed6758`. The CLI and authenticated Research OS read a bounded graph slice and return deterministic, ranked unverified proposals. The six generators cover comparable conflicts, evidence gaps, profile boundaries, negative results, method composition and candidate transfers between declared domains. Proposals carry pinned source revisions, counterevidence when found, alternatives, assumptions, limitations, a falsification test, suggested metrics and checked planning costs. The [ranking decision](docs/decisions/ADR-016.md) and [commands](docs/research/APOLLO_EXECUTION_PLAN.md#apo-03-hypothesis-discovery) describe their use.

The DAE-04 adapter indexes all 20 matched development cases at eight evaluations per arm from the digest-checked archive. Adaptive hybrid won 2, tied 13 and lost 5 against greedy. The polarity changes occurred across distinct cases. The engine treats them as boundary and negative-result leads and also proposes a factorial method test. The archive is one provenance group. These findings provide no confirmed contradiction, equivalence claim or estimate for a broader population.

The fixture scanned 23 nodes and 22 links and returned three proposals in order. Negative-result review scored 15 and estimates 320 candidate evaluations. The boundary test scored 12 and estimates 384 evaluations. Method composition scored 11 and estimates 48 benchmark trials. Two fresh CLI processes produced byte-identical JSON with SHA-256 `db6a5053cdd4fd4901edf0ca3cbb75f287c724f96530b80a3725b662d4aff541`. The graph digest matched the corresponding export. The counts and scores describe the frozen development input and advisory rubric, not measured performance on a new experiment.

Fixtures exercise mismatched scopes, dependent copies, conflicting annotations, unsupported text, duplicate designs, misleading domain associations and incomplete literature coverage. Ranking is a prioritization heuristic. Runtime costs and scientific validity require a separately reviewed test plan. PDF ingestion and live provider qualification remain pending.

Validation passed 27 API unit tests, 14 platform tests, strict API Clippy, formatting, the offline release build, five frontend tests, type checking, lint, production web build, desktop and mobile browser smoke, and documentation and schema checks. The browser opened a ranked proposal and its source and counterevidence panels. Frozen research artifacts and the held-out protocol match the APO-02 checkpoint.

## APO-02 persistent scientific knowledge graph

Built from `3bac54c` using the existing literature SQLite database and typed campaign snapshot. Canonical paper aliases and normalized keys identify entities. Revisions and opposing assertions are retained. Edges pin both endpoint revisions. Literature imports project metadata and exact passages atomically. The operator can classify a checked passage as a reported claim. Qualified verification receipts are copied only from the trusted campaign store and marked at capture.

The authenticated Research OS Knowledge Graph view supports search, bounded traversal, citation and dependency exploration, revision history and duplicate groups. Exported slices carry a reproducible digest. Commands, identity and authority rules are in the [APOLLO plan](docs/research/APOLLO_EXECUTION_PLAN.md) and [graph decision](docs/decisions/ADR-015.md).

PDF ingestion and live provider qualification remain pending. The two-paper real corpus tests exact passage grounding under CC BY 4.0. Synthetic fixtures test contradictory outcomes, cycles, identity changes, malformed and malicious input. These cases do not establish extraction accuracy on an unreviewed corpus. [Corpus and attribution](research/literature/README.md#apo-02-reviewed-excerpts)

A synthetic star with 65 nodes and 64 links returned 64 nodes and 63 links at depth one with explicit truncation. Five fresh debug CLI processes emitted identical 81,258-byte export files. Independent SHA-256 over compact graph JSON matched `521ddf30d939acf87cb96d084d2288cadc317894b0facf863f2d1160c73b3137`. Whole-process median was 714.1 ms including startup and database opening during package validation. SQLite query plans used the type and title index and the inbound edge index. The measurement covers one small fixture on the development host.

Validation passed 22 API unit tests, 13 platform tests, strict API Clippy, the offline release build, five frontend tests, type checking, lint, the web build, and desktop and mobile browser smoke. The browser followed a captured campaign edge into a claim dependency. Documentation and schema validation passed. Frozen research results and archived documents match the APO-01 checkpoint.

## APO-01 scientific literature intelligence

Started from `a18c1b8` (`daedalus-engineering-verified`). APOLLO extends the existing Rust provider adapters and SQLite catalog with bounded page caching, persisted provider cooldowns, deduplicated metadata snapshots, explicit arXiv revisions and source-reported citation identifiers. The first planning citation remains stable. The Research OS Literature view shows snapshots, source links, and extracted prose with byte offsets when a local text is present.

The CLI accepts authorized UTF-8 text under the rights manifest in [ADR-014](docs/decisions/ADR-014.md). It archives exact bytes and uses deterministic heading recognition for questions, methods, claims, results and limitations. The archive records missing sections and truncation. Extracted passages carry unverified status and cannot affect approvals, execution or trusted evidence. The [starter corpus](research/literature/README.md) contains three real metadata records and a separate synthetic full-text fixture.

Provider behavior is checked through deterministic fixtures. Upstream index changes can shift offset pages after cache expiry. PDF conversion, larger corpora with reviewed rights, and extraction quality measurement are follow-up work in the [APOLLO execution plan](docs/research/APOLLO_EXECUTION_PLAN.md).

Validation passed 18 API unit tests, 12 platform tests, strict API Clippy, the offline release build, five frontend tests, TypeScript, lint, the web build, and desktop/mobile browser smoke against the release binary. Documentation and schema checks passed. Fresh CLI processes reopened all three starter metadata revisions and resolved PaperBench through its existing DOI alias. Frozen research records, Rune and the workspace lockfile matched the DAEDALUS checkpoint.

## DAE-10 final DAEDALUS engineering release

Release candidate began at `3af6dbd`, the separate Rune probe checkpoint. Rune remains optional, unqualified and outside the default Cargo workspace and release gates. The starter build and host memory interruption are recorded in `experiments/rune-native/README.md`; the probe is not an AETHER interface.

Windows x64 release validation passed with Cargo limited to two jobs: workspace tests (77 passed, 9 ignored), strict workspace Clippy, formatting and locked offline workspace release build. Frontend validation passed after `npm ci --ignore-scripts`: five tests, TypeScript, Biome lint and production build. `node tests/browser-smoke.cjs` passed desktop and 390-pixel mobile flows, including all seven DAE-08 pages and existing campaign workflows. The documentation suite passed 3 timestamp contract tests, 13 schemas, 65 JSON files, 271 Markdown files, 1,287 relative destinations, 41 mapped requirements and five negative cases.

Authentication, worker admission/recovery, Windows Job Object quota, process, timeout, cancellation, child containment and environment tests passed as part of the workspace suite. Discovery and inference integration tests exercised fresh persisted artifacts; no development benchmark was repeated. Frozen `research/results` and `docs/archive` are unchanged from the DAE-08 baseline (`git diff --exit-code 60830db -- research/results docs/archive` passed). The ECAR held-out study was not executed. The worker still has no OS filesystem denial, network denial or restricted identity; it is not a complete sandbox. No external dependencies, models or system settings were added. The validated source is tagged locally as `daedalus-engineering-verified`; nothing was pushed or deployed.

## DAE-09R Rune feasibility — interrupted

Starting from `dbc9e82`, inspected upstream Rune at `66006728af7cd3dc54a6bb31ca7690242fb8d6aa`, its required patches, examples, documentation and notices. The isolated [build probe](experiments/rune-native/README.md) contains only the upstream hello starter; it is not an AETHER prototype. Cargo resolved 435 packages. A single-job Windows build was stopped while compiling `windows 0.62.2` after observed free physical memory fell to 122,684 KiB; rustc used approximately 2.09 GB working set. Separate workspace metadata passed. No executable, headless screenshot, API integration or runtime measurements were obtained. Decision is provisionally **defer**, pending a build with sufficient available memory and the requested functional and performance validation. No commit is claimed for this incomplete work.

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

## DAE-05 reliability-aware ECAR development experiment

The Athena pilot exposed opposing graph effects in the valuable and unreliable hubs. This experiment compares full downstream mass, a fixed half-weight prior, and evidence-updated mass while holding each experimental arm's information bonus and contradiction handling constant. Multiplying the whole score by reliability would also suppress direct utility and could starve uncertain, consequential checks. The implemented variant discounts only downstream utility. It treats usable independent receipts as support or contradiction, starting from Beta(1,1). Transport or execution failures consume budget and contribute no receipt. The posterior mean estimates the support rate of a future usable receipt. The model does not infer execution success or calibrated ground-truth correctness.

One-step expected Beta variance reduction, scaled to equal claim mass at cold start, boosts verification according to the full attainable claim mass. A provisional contradiction changes priorities. A trusted rejection prunes the dependent subtree. The existing scheduler still applies eligibility, checked reserves, normalized cost and deterministic ties. The independent simulated evaluator alone issues verdicts. Hidden truth and noise are confined to the synthetic environment and outcome scoring. The approximation assumes exchangeable receipts and a topologically ordered single-parent forest. It applies the hub estimate once to its attainable descendants and does not infer reliability along each edge.

Build `cargo build --locked --release -p aether-engine --example ecar_reliability --example ecar_diagnostics`. Run `target/release/examples/ecar_reliability` without arguments to emit the [raw report](research/results/dae-ecar-reliability-development.json). The [embedded grid](benchmarks/dae-ecar-reliability-development.json) crosses five development families, 24 seeds, and caps 12, 24, 48 and 84. Common draws omit arm and cap. Fresh processes matched all 2,880 outcome rows. The fixed flat, adaptive flat and ECAR rows matched all 1,440 corresponding rows from `ecar_diagnostics --development-v2 benchmarks/dae-ecar-reliability-development.json`. The raw file SHA-256 is `be989ab7242815b36c5a9dda0839e552928783aa25e69ba72b36a47cda22f46a`.

| Policy | Mean true verified utility (%) | Verified / false | Mean scheduling ns per call |
|---|---:|---:|---:|
| Fixed flat | 9.563 | 366 / 1 | 3,852 |
| Adaptive flat | 13.973 | 520 / 2 | 3,714 |
| Original ECAR | 13.541 | 574 / 2 | 3,609 |
| Full mass | 12.105 | 542 / 2 | 3,942 |
| Fixed prior | 12.088 | 540 / 2 | 3,786 |
| Posterior mean | 12.088 | 540 / 2 | 3,827 |

Posterior weighting tied the fixed prior on utility in all 480 cases despite 55 changed action traces. It tied full mass in 478 and lost two sparse-evidence cases, for −0.018 points overall. Relative to original ECAR, it gained 3.241 points in unreliable hubs but lost 1.454 overall, 3.216 in valuable hubs, 0.951 in misleading dependencies, and 2.721 at the tightest cap. That comparison includes multiple policy changes, so it does not identify a causal reliability effect. Timings include score preparation and scheduling, exclude simulated experiment cost and reporting, and come from one fixed-order Windows run. The next development comparison should isolate the information bonus and collect more observations per claim before assessing adaptive weighting again.

Twenty-three targeted tests, strict engine Clippy, formatting, the release example builds and frozen artifact checks passed. The archived pilot estimate and held-out protocol remain in place. [Execution queue](EXECUTION_QUEUE.md)

## DAE-05B reliability and information bonus ablation

The archived DAE-05 posterior and fixed-prior arms changed 55 of 480 action traces. In 54 of those cases they ran the same actions in a different order. All 55 preserved the verified and false-verified counts. The original stage rules allow at most two receipts before the final challenge. This explains why ranking changes often had no effect on the utility reached within the cap.

Build `cargo build --locked --release -p aether-engine --example ecar_reliability` and run `target/release/examples/ecar_reliability --ablation`. The predefined grid uses the five DAE-05 development families, seeds 301–306, caps 24 and 84, and receipt noise of 10% and 30%. Exogenous histories contain 0, 2 or 8 source groups per claim. The 8-group condition is also repeated with four exact copies per group. Each consistent group contributes one vote and mixed signs within a group abstain. History acquisition is outside the matched scheduling budget and supplies no verdict evidence. Scheduled observations, shared random draws and verdict rules match across arms. A separate usable receipt is drawn after scheduling for each claim to check calibration.

At Beta(1,1), the grid crosses direct-only, full downstream mass, fixed half-weight and posterior weight with information multipliers 0, 1 and 2. Beta(4,4) and Beta(12,12) compare fixed and posterior weights with the bonus off and on. Both arms can update the information term with receipts when it is enabled. With the bonus off, the fixed versus posterior contrast isolates learning in the downstream discount. Rankings compare affordable action order on shared histories up to the first divergent action. Reported utility is true verified weight divided by total claim weight. Calibration uses terminal posterior probabilities for next-receipt support, Brier score and five fixed-width calibration bins. The fixed forecast of 0.5 has Brier 0.25.

| Controlled contrast | Utility change (points) | Wins / ties / losses | Shared-state ranking changes |
|---|---:|---:|---:|
| Posterior vs fixed prior at Beta(1,1), bonus off | +0.0004 | 3 / 354 / 3 | 196 |
| Posterior vs fixed prior at Beta(1,1), bonus 1 | +0.0123 | 2 / 358 / 0 | 152 |
| Full mass vs direct-only at Beta(1,1), bonus off | +0.5709 | 54 / 306 / 0 | 288 |
| Fixed half-weight vs full mass at Beta(1,1), bonus off | −0.0185 | 0 / 357 / 3 | 144 |
| Posterior bonus 1 vs off at Beta(1,1) | +0.2226 | 11 / 349 / 0 | 269 |
| Posterior bonus 2 vs off at Beta(1,1) | +0.7644 | 25 / 333 / 2 | 318 |
| Posterior bonus 1 vs off at Beta(4,4) | −0.2773 | 1 / 354 / 5 | 223 |
| Posterior bonus 1 vs off at Beta(12,12) | 0 | 0 / 360 / 0 | 76 |

Each contrast pools 360 matched cases and excludes the duplicate histories. All 2,400 cloned-history pairs matched votes, action rankings, outcomes and probes. At the reference posterior setting, eight groups at 10% noise moved the initial mean 0.323 from 0.5 on average. Subsequent scheduling moved it 0.020 on average, versus 0.112 in the cold-start condition. Brier improved from 0.194 to 0.106. At 30% noise it improved from 0.253 to 0.228, while five-bin calibration error worsened from 0.064 to 0.097. Rich history at 30% noise used 40.27 ms per case versus 39.00 ms at cold start and retained three false verifications in both cases. Better forecasts did not translate into more verified utility. At Beta(1,1) and bonus 1 the posterior scheduler averaged 3.48 µs per call versus 3.58 µs for the fixed prior. These are single host measurements excluding rank logging and simulated task execution.

The [compact sensitivity results](research/results/dae-ecar-reliability-ablation.json) retain all settings, paired contrasts, calibration, false verification, resource use and scheduler timings. Two release processes reproduced all 9,600 outcomes. Their canonical outcomes SHA-256 is `cd1bcfb1513abc9d0a201324f7403d66da400a7fc5041b0017af05da8101441a` using UTF-8 JSON with sorted object keys and separators `,` and `:`. The unflagged run reproduced all 2,880 DAE-05 rows. Twenty-five targeted tests, strict Clippy, formatting and the release build passed. The experiment uses synthetic receipts and exact copies. A future development grid should include partial dependence and independently selected task families before choosing an information bonus. [Execution queue](EXECUTION_QUEUE.md)

## DAE-06 adaptive test-time compute

DAE-05B is frozen at `170d354`. Its compact results retain SHA-256 `7b013fcc37704818bc25c4d23f56df9cda35ebd68586223fe8679a2ee17823ee`. Partially dependent evidence sources remain future research.

`aether-lab inference-study fixture` compares single-response, best-of-three, fixed refinement and adaptive inference on six-element development maximum-subarray inputs. `inference-study ollama` runs one matched input with the pinned local Qwen model and checks that Ollama reports cloud disabled before submitting a prompt. The model receives the public list and, for critique or refinement, the previous typed interval. The prompt has no reference score, trusted verdict or final-test data. A reviewed worker supplies the baseline and the independent evaluator checks model bounds and the exact optimum. Only a passing authoritative check selects an answer.

Each trial has separate checked limits of three model calls, 6,336 model tokens, three verifier calls and one native execution. Model requests cap generation at 64 tokens with a 2,048-token context. Each trace records prompt and output counts and a reasoning subset when available. The fixture uses scripted usage, while Ollama reports prompt and output counts with thinking disabled and no separate reasoning-token counter. Unknown usage and failed requests charge their full reservation. The controller discards over-limit responses, deduplicates intervals, and keeps candidate lineage. Adaptive inference checks new candidates before spending more inference and stops on success, repeated non-progress, cancellation or exhaustion.

Every completed trial, including abstentions, is stored by SHA-256 as development evidence in the existing SQLite store. Ctrl-C cancels at controller boundaries and the reviewed worker observes the cancellation flag. A blocking local request can finish or time out before cancellation is recorded. Provider failures recover within the remaining call budget. An abruptly terminated process can be rerun from the original fixture, with prior completed trials retained. The interrupted in-flight trial is not resumed.

Two release fixture processes reproduced the decisions, lineage and verdicts across 12 trials. The fixture reports 80 prompt and 24 output tokens per call by construction.

| Fixture strategy | Verified / 3 | Model calls | Prompt / output tokens | Independent checks | Native execution ms |
|---|---:|---:|---:|---:|---:|
| Single response | 0 | 3 | 240 / 72 | 3 | 145 |
| Best-of-three | 1 | 9 | 720 / 216 | 3 | 115 |
| Fixed refinement | 0 | 9 | 720 / 216 | 3 | 126 |
| Adaptive | 1 | 8 | 640 / 192 | 7 | 112 |

[Raw fixture traces](research/results/dae-inference-fixture.json)

The owned Ollama service qualified with cloud disabled and the existing Qwen 3.5 4B Q4_K_M manifest `e0b8c85c75aad2573ff49bec177ebb1013772fba665a53d64c04266439a2222d`. All four strategies saw the same seed-19 list. All ten calls returned the same distinct interval, with sum −65 against an exact optimum of 84. The verifier rejected every strategy. Adaptive stopped after two repeated proposals. The pilot used 821 prompt tokens and 210 output tokens. The provider did not report a separate reasoning count. Inference times include preflight and the first cold load, so they do not establish a timing ranking.

| Live strategy | Verified / 1 | Calls | Prompt / output tokens | Inference ms | Verify / execution ms |
|---|---:|---:|---:|---:|---:|
| Single response | 0 | 1 | 77 / 21 | 13,112 | 1 / 85 |
| Best-of-three | 0 | 3 | 231 / 63 | 3,420 | 1 / 59 |
| Fixed refinement | 0 | 3 | 253 / 63 | 3,229 | 1 / 44 |
| Adaptive | 0 | 3 | 260 / 63 | 3,251 | 1 / 47 |

The 16 saved report digests resolved to their exact evidence bytes in SQLite. The raw fixture and [local Qwen traces](research/results/dae-inference-local-qwen.json) have SHA-256 `e44d0d8d2d92b838aaa940b0f1d26d51ba8c9dc2b2381dbf5c49c334b096c636` and `16df85c7226ee8deca8de870da895ac85710b742b622f4444cabee27ac2cd447`. Twenty-six API tests, strict API Clippy, formatting and the release build passed. One local model, one live task and one strategy order limit the pilot. Before comparing policies again, define a larger development grid and rotate order to separate cold-load effects. [Execution queue](EXECUTION_QUEUE.md)

## DAE-07 native performance laboratory

Build the pinned offline release binary with `cargo build --locked --offline --release -p aether-api --bin aether-lab`, then run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-native-performance.ps1`. The script runs only public synthetic workloads and reviewed native worker tasks, launches a temporary loopback API, records exact benchmark order and raw operation samples, and writes [the run record](research/results/dae-native-performance.json). It adds host telemetry without a new dependency. Samples use five warmups per case; execution order is deterministically shuffled per round. The record reports p50/p95, operations per second and a 1,000-resample percentile 95% bootstrap interval for the median. p99 is intentionally null below 1,000 observations.

The observed host was Windows 11 Home Single Language 10.0.26300, Intel i5-13420H, 12 logical processors and 15.64 GiB visible RAM. Release build used rustc 1.96.1, `x86_64-pc-windows-msvc`, LLVM 22.1.2. The 100-sample operations report p50/p95 in microseconds: ECAR scheduling 32 actions 37.5/66.4 and 128 actions 124.2/256.9; graph snapshots 64 claims 2,282.9/3,235.2 and 256 claims 4,245.4/5,444.4; evolution 24 evaluations 60.0/116.2 and 96 evaluations 165.9/246.7; durable SQLite append 14,195.7/19,331.9; authenticated loopback API list 5,375.2/7,212.5; report generation 9,798.9/12,347.0. The pure adaptive action-selection function measured 0.004/0.006 microseconds per decision. Reviewed worker lifecycle used 40 samples at 51,290.4/58,339.9 microseconds. Four-way threaded discovery used 20 batch samples at 1,019.6/1,307.9 microseconds and 3,822.8 operations/s. These are one-host observations, not comparative claims or regression thresholds.

The measured phase took 5.684 seconds; the outer process took 10.827 seconds including startup, fixture setup and warmups. PowerShell sampled the benchmark controller at 20 ms and recorded 4 CPU seconds, 21.18 MB peak working set and 12.93 MB peak private bytes. This CPU counter is coarse and excludes the separate API and worker child processes; worker and API latency are reported separately. Model inference was excluded, and adaptive inference reports action-selection overhead only. Held-out ECAR was not executed. No optimization was applied: these measurements alone do not identify a profiled code hotspot. The [raw run record](research/results/dae-native-performance.json) has SHA-256 `5ce19de59bd84423c232568ea868efa4ffb7a31f83336c22be93124caf0cd3f4`. Frozen DAE-05B, fixture inference and local Qwen report hashes remained `7b013fcc37704818bc25c4d23f56df9cda35ebd68586223fe8679a2ee17823ee`, `e44d0d8d2d92b838aaa940b0f1d26d51ba8c9dc2b2381dbf5c49c334b096c636` and `16df85c7226ee8deca8de870da895ac85710b742b622f4444cabee27ac2cd447`. Sixteen API unit tests and eleven platform tests, strict API Clippy, formatting, release build and documentation validation passed. [Execution queue](EXECUTION_QUEUE.md)

## DAE-08 AETHER Research OS

Reworked the existing React console into a local scientific command centre with Mission Control, Evidence Explorer, Discovery Laboratory, Intelligence Observatory, Compute Observatory, Research Archive and Security Centre. Existing campaign, evidence graph, reporting, planner, benchmark, runtime and literature workflows remain available. Historical DAE-07 samples are explicitly labeled as recorded measurements; local inference is historical, and the interface does not imply a live model or telemetry feed. The archive retains negative and null outcomes and presents the Athena pilot's recorded +0.173 utility points with its bootstrap interval (−0.840 to +1.188), equivalent to the reported +0.17 [−0.84, +1.19]. No held-out ECAR execution occurred.

Added bearer-authenticated, read-only archive routes. The index is derived from build-pinned result records; raw downloads return original bytes, carry their SHA-256, and the browser verifies the digest before saving. Integration coverage confirms unauthenticated denial, allowlisted artifact retrieval, exact-byte hashing and unknown-ID rejection. Frozen result hashes remain unchanged: DAE-07 `5ce19de5…caf0cd3f4`; DAE-02/03 `94c955c6…3f898c88fc5d`; DAE-04 `bc3654d2…256736ced`; DAE-05B `7b013fcc…e17823ee`; Athena pilot `6c7026fd…840a2e190`; DAE-06 fixture `e44d0d8d…b096c636`; local Qwen `16df85c7…ac2cd447`.

Validation: all 27 `aether-api` tests passed (16 unit, 11 platform); five frontend tests, TypeScript, Biome lint, and production build passed. Chromium desktop and mobile smoke passed for all seven new pages and the existing campaign workflows. `cargo fmt --all` and `git diff --check` passed. No new dependency or benchmark run was added. The Windows worker still lacks filesystem, network and restricted-identity isolation; it is supervised process execution, not a complete sandbox. The repository remains local and the held-out study deferred. [Execution queue](EXECUTION_QUEUE.md)

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
