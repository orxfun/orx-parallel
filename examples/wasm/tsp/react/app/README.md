# Vite + React host app for `orx-parallel`

This demo uses **Vite** with **React + TypeScript** to drive a wasm-backed TSP search UI.

It illustrates enabling parallel computation using `orx-parallel` with wasm.

## wasm bindings

The wasm package is built into `./pkg` with the `build:wasm` script:

- `wasm-pack build ../wasm_bindings --target web --out-dir ../app/pkg`
- the Rust build uses nightly and enables shared-memory/threaded wasm flags

The app imports the generated bindings from `./pkg/wasm_bindings.js`:

- call `init()` once before using wasm exports
- use `locations(seed, num_cities)` to generate the TSP points
- use `run_search(...)` for the actual computation in the worker

## build:wasm

The `build:wasm` script in `package.json` is the step that produces the generated bindings and the `.wasm` artifact used by the app.

Note that `app` is the name of this directory, so the `../app/pkg` output path in the script resolves to the `pkg` folder next to this README.

Important parts of that script:

- `RUSTUP_TOOLCHAIN=nightly` selects nightly Rust because the wasm build uses unstable `-Z build-std`
- `wasm-pack build ../wasm_bindings --target web --out-dir ../app/pkg` compiles the Rust crate for the browser and writes the generated JS bindings into `./pkg`
- `-C target-feature=+atomics`, `--shared-memory`, and the TLS exports enable shared-memory wasm, which is required for browser threads
- `--import-memory` and `--export=__wasm_init_tls` / `__tls_*` are part of the threaded wasm setup used by this example

When you change the Rust side, rerun `npm run build:wasm` so the generated `./pkg/wasm_bindings.js` and `./pkg/*.wasm` stay in sync with the code imported by the app.

## React component layout

The UI is intentionally split into composable components under `src/components/`:

- `ControlsSection.tsx` handles inputs and actions
- `StatusSection.tsx` renders the running overlay and metrics
- `CanvasView.tsx` renders points and the best tour
- `CodeCard.tsx` renders highlighted sequential/parallel code snippets

`src/App.tsx` owns state and orchestration, similar to the role of the Leptos component crate in the sibling example.

## search worker

`src/search-worker.ts` is a module worker created through `src/search-runner.ts` with `new Worker(new URL("./search-worker.ts", import.meta.url), { type: "module" })`.

Inside the worker:

- import `init`, `init_wasm_parallel_runtime`, and `run_search` from `./pkg/wasm_bindings.js`
- call `init()` in the worker before touching wasm exports
- call `init_wasm_parallel_runtime(threadCount)` before the first parallel search in that worker
- send search results back to the main thread with `postMessage`

In this example, the thread pool is created per worker. That means each worker owns its own pool, and a new worker implies a new pool.

You can also keep a persistent search worker alive and reuse it for multiple searches. Either way, the worker must initialize the parallel runtime once before it runs parallel search.

## config files that matter

- `package.json` defines useful scripts:
  - `dev:full` builds wasm first, then starts Vite
  - `dev` starts Vite only
  - `build` runs TypeScript typechecking before the production build
- `vite.config.ts` enables `@vitejs/plugin-react`, keeps `worker.format: "es"`, and sets COOP/COEP headers
- `tsconfig.json` uses JSX + bundler-friendly TypeScript settings so the generated wasm package and Vite imports typecheck cleanly

Importantly, parallel computation will not work without `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers. As mentioned above, this is added to Vite configuration. If you serve the built `dist/` folder outside Vite, the server must send the same COOP/COEP headers. A plain static server like `npx serve dist` will not work for threaded wasm because the browser will reject `SharedArrayBuffer` unless `self.crossOriginIsolated` is true. This repo includes `npm run serve:dist`, which serves `dist/` locally with the required headers.

## minimal flow to run locally

Vite:

```bash
npm run dev:full

# or
npm run build:wasm
npm exec -- vite
```

Or using the example static server:

```bash
npm run build:wasm
npm run build
npm run serve:dist
```
