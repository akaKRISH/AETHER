# AETHER Research Console

The console is a React and TypeScript client for the local Rust API. Campaigns, experiments, artifacts, claims, budget usage, events, benchmarks, runtime information, literature and reports come from the service.

## Run locally

1. Start the Rust API on loopback with `AETHER_TOKEN` configured.
2. In this directory run `npm ci` and `npm run dev`.
3. Open the Vite address, select **Connect API**, and enter the server token. The token is kept in the current tab's memory. Disconnecting or reloading clears it.

The dev server proxies relative `/v1` requests to `http://127.0.0.1:8080` by default. Set `AETHER_WEB_API_ORIGIN` in the Vite process environment to use another local API address. For deployed static assets, serve `dist/` and the `/v1` API on one origin.

Create a draft, review the frozen protocol hash, approve it, then start the campaign. The active campaign snapshot updates every 2.5 seconds. Select claim or evidence nodes by mouse or keyboard to inspect references. Benchmarks display the service's evidence scope. Reports can be downloaded as JSON.

`npm run check` checks TypeScript. `npm run build` checks types and generates the static distribution.
