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

## Athena campaign result

On 25 November 2026, the owner approved protocol SHA-256 `3906c69297b1d1582c6f85cb35f44876e5f0d38c7c588e6ea877bfa31bb53dc9`. Before approval, the pinned protocol matched the objective task pack, chose the registered insertion sorter, used seed 19 and 96 items, and preserved five attempts and a 12,000 ms cap. The pinned executable SHA-256 remained `e79057c75e9a04ec9a48a951c0662669d609518fa52b9d57046ff9b0f044daa2`.

The approved run completed three successful attempts within its budget: exploration, fresh-process replay and challenge. Trusted exact-reference fractions were 1.0 on both 96-item seeds, 19 and 2236246020. The replay matched the exploration input digest, while the challenge input digest differed. The frozen claim was verified by the pinned evaluator and protocol rule. The additional follow-up claim remains proposed.

The report is `research/results/athena-local-campaign.json`, SHA-256 `7b1d9d9de5784b830d864cee0930ecc8f30c6dcc2cb28c5c697d4cba775e6ad2`. A separate report export matched it byte for byte. The attempt budget recorded three of five and 140 of 12,000 ms. Model inference happened during planning and used its separately recorded provider counters.

## Exact next action

ATH-02 is **BLOCKED** at the research review gate. The development runner rejects the frozen held-out phase. Different seeds and streams share random draw keys, and the summary can accept incomplete case grids. The public specification exposes the simulation inputs. Before any held-out run, fix key separation and case validation, freeze the approved execution path, and record independent custody for a blinded confirmatory study. The pilot remains +0.17 utility points with its reported 95% interval of −0.84 to +1.19. [Review and exact execution instructions](docs/research/ATH_02_INDEPENDENT_ECAR_REVIEW.md)

## Remaining dependencies

The desktop Ollama process still reports cloud enabled. An owned process with the existing local model verified cloud disablement and completed calibration on the RTX 3050 at two confirmed context lengths. [Raw measurements](research/results/athena-ollama-contexts.json)

The approved-task Windows profile has process lifecycle containment. OS-enforced CPU and memory ceilings, filesystem and network boundaries, and restricted identities require separate qualification. Formal held-out testing requires independent review and task custody. See [next actions](NEXT_ACTIONS.md).
