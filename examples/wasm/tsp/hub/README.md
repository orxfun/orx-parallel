# WASM TSP Hub

This page gives one central entrypoint for the four framework demos:

- vanilla
- react
- yew
- leptos

## How it works

The hub has 4 buttons.
Clicking a button navigates this tab to a bundled app page.

This avoids cross-origin-isolation issues that can happen with SharedArrayBuffer-based wasm-thread apps inside iframes.

Bundled paths:

- ./apps/vanilla/index.html
- ./apps/react/index.html
- ./apps/yew/index.html
- ./apps/leptos/index.html

## Build one site to host

Run:

```bash
cd examples/wasm/tsp/hub
./build-hub.sh
```

This command:

- builds vanilla, react, yew, and leptos apps
- copies their dist outputs into hub/apps/<framework>
- copies their dist outputs into hub/site/apps/<framework> (for deployable static site)
- copies hub/index.html into hub/site/index.html

Do not open hub pages with file://. Use an HTTP server.

## Local run

Serve hub/site with an HTTP static server.

Option A:

```bash
cd examples/wasm/tsp/hub
npm run serve
```

Option B:

Example using Python:

```bash
cd examples/wasm/tsp/hub/site
python3 -m http.server 8080
```

Open:

- http://localhost:8080

## Deployment note for wasm threads

These apps use shared-memory wasm threads. Your static host must send these headers for the hub page and app pages:

- Cross-Origin-Opener-Policy: same-origin
- Cross-Origin-Embedder-Policy: require-corp

Without these headers, browser thread features used by the apps will be blocked.
