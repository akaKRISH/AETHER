# Tests

The [test plan](../docs/testing/TEST_PLAN.md) maps planned checks to requirements. The [results](../docs/testing/TEST_RESULTS.md) record the checks that ran. Use synthetic data and temporary workspaces.

`cargo test --locked --workspace` exercises the native fixture, store, scheduler, worker and loopback API. Run `npm --prefix web test` for the HTTP client tests.

To reproduce the browser smoke, first build `cargo build --locked --release --workspace`, then run `npm --prefix web ci` and `npm --prefix web run build`. Run `node tests/browser-smoke.cjs` with Playwright and Chromium installed. Set `AETHER_PLAYWRIGHT_PATH` to an installed Playwright module if it is outside the normal Node lookup paths, and `AETHER_CHROMIUM_PATH` to use an existing Chromium executable. The smoke creates and removes its own temporary state and starts its own loopback server on a free port. Set `AETHER_SCREENSHOT_DIR` to retain the desktop evidence and verification views and the mobile overview.

Run `npm --prefix web run lint` for the frontend lint gate. After the locked release build, run `node tests/measure-loopback.cjs` for local route timing. Criterion commands are `cargo bench --locked -p aether-engine --bench scheduler` and `cargo bench --locked -p aether-store --bench evidence`. [Recorded timings](../docs/research/BENCHMARK_RESULTS.md)
