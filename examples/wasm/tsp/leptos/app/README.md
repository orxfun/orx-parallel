# Vite host app for the Leptos `orx-parallel` demo

This demo uses **Vite** with **TypeScript** to host a wasm-built Leptos UI.

It illustrates enabling parallel computation using `orx-parallel` with wasm.

## generated wasm UI package

The wasm package is built into `./pkg` with the `build:wasm` script:

- `wasm-pack build ../components --target web --out-dir ../app/pkg`
- the Rust build uses nightly and enables shared-memory/threaded wasm flags

The browser app imports the generated bindings from `./pkg/components.js`:

- call `init()` once before using wasm exports
- call `start_app()` to mount the Leptos UI
- the worker also imports `init_parallel_runtime(...)` and `run_search(...)` from the same generated package

## build:wasm

The `build:wasm` script in `package.json` is the step that produces the generated bindings and the `.wasm` artifact used by the browser app.

Note that `app` is the name of this directory, so the `../app/pkg` output path in the script resolves to the `pkg` folder next to this README.

Important parts of that script:

- `RUSTUP_TOOLCHAIN=nightly` selects nightly Rust because the wasm build uses unstable `-Z build-std`
- `wasm-pack build ../components --target web --out-dir ../app/pkg` compiles the Rust UI crate for the browser and writes the generated JS bindings into `./pkg`
- `-C target-feature=+atomics`, `--shared-memory`, and the TLS exports enable shared-memory wasm, which is required for browser threads
- `--import-memory` and `--export=__wasm_init_tls` / `__tls_*` are part of the threaded wasm setup used by this example

When you change the Rust side, rerun `npm run build:wasm` so the generated `./pkg/components.js` and `./pkg/*.wasm` stay in sync with the code imported by the app.

## app bootstrap

`src/main.ts` is the browser entrypoint.

Inside it:

- import `init` and `start_app` from `./pkg/components.js`
- import `./search-runner.ts` so the Leptos UI can call the worker bridge through `globalThis.runSearchAlgorithm`
- call `init()` before `start_app()` so the generated wasm package is ready before the UI mounts

## search worker

`src/search-worker.ts` is a module worker created by `src/search-runner.ts` with `new Worker(new URL("./search-worker.ts", import.meta.url), { type: "module" })`.

Inside the worker:

- import `init`, `init_parallel_runtime`, and `run_search` from `./pkg/components.js`
- call `init()` in the worker before touching wasm exports
- call `init_parallel_runtime(threadCount)` before the first parallel search in that worker
- send search results back to the main thread with `postMessage`

In this example, the worker is short-lived and created per search. That means each worker owns its own pool, and a new worker implies a new pool.

You can also keep a persistent search worker alive and reuse it for multiple searches. Either way, the worker must initialize the parallel runtime once before it runs parallel search.

## config files that matter

- `package.json` defines the useful scripts:
  - `dev:full` builds wasm first, then starts Vite
  - `dev` starts Vite only
  - `build` creates the production bundle
  - `preview` builds and serves the Vite preview server
- `vite.config.ts` uses `worker.format: "es"` so the module worker is emitted in ESM form
- `vite.config.ts` also sets `server.headers` for `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy`; this is required for cross-origin isolation, which in turn is required for shared-memory wasm and browser threads
- `tsconfig.json` uses bundler-friendly TypeScript configuration so the generated wasm package and Vite imports typecheck cleanly

Importantly, parallel computation will not work without `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers. As mentioned above, this is added to Vite configuration. If you serve the built app outside Vite, the server must send the same COOP/COEP headers. A plain static server like `npx serve dist` will not work for threaded wasm because the browser will reject `SharedArrayBuffer` unless `self.crossOriginIsolated` is true.

## minimal flow to run locally

Vite:

```bash
npm run dev:full

# or
npm run build:wasm
npm exec -- vite
```

Preview:

```bash
npm run build:wasm
npm run build
npm run preview
```