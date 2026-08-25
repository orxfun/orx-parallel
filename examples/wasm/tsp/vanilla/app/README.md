# Vite + TypeScript UI for `orx-parallel`

This demo uses **Vite** with **TypeScript** to drive a wasm-backed TSP search UI.

It illustrates enabling parallel computation using `orx-parallel` with wasm.

## wasm bindings

The wasm package is built into `./pkg` with the `build:wasm` script:

- `wasm-pack build ../wasm_bindings --target web --out-dir ../app/pkg`
- the Rust build uses nightly and enables shared-memory/threaded wasm flags

The UI imports the generated bindings from `./pkg/wasm_bindings.js`:

- call `init()` once before using wasm exports
- use `locations(seed, num_cities)` to generate the TSP points
- use `run_search(...)` for the actual computation

## build:wasm

The `build:wasm` script in `package.json` is the step that produces the generated bindings and the `.wasm` artifact used by the UI.

Note that `app` is the name of this directory, so the `../app/pkg` output path in the script resolves to the `pkg` folder next to this README.

Important parts of that script:

- `RUSTUP_TOOLCHAIN=nightly` selects nightly Rust because the wasm build uses unstable `-Z build-std`
- `wasm-pack build ../wasm_bindings --target web --out-dir ../app/pkg` compiles the Rust crate for the browser and writes the generated JS bindings into `./pkg`
- `-C target-feature=+atomics`, `--shared-memory`, and the TLS exports enable shared-memory wasm, which is required for browser threads
- `--import-memory` and `--export=__wasm_init_tls` / `__tls_*` are part of the threaded wasm setup used by this example

When you change the Rust side, rerun `npm run build:wasm` so the generated `./pkg/wasm_bindings.js` and `./pkg/*.wasm` stay in sync with the code imported by the UI.

## persistent search worker

`src/search-runner.ts` owns one persistent `ParallelWorker` from
`orx-parallel-web`. It initializes the generated wasm package and its parallel
runtime once, then sends every `run_search(...)` call through the same worker.
Calls are serialized, which matches the wasm pool's single active computation
scope while retaining parallelism inside each search.

The worker terminates when the page unloads. The Vite plugin rebuilds the wasm
package, prepares the nested wasm workers, and provides the cross-origin
isolation headers required for shared-memory wasm.

## config files that matter

- `package.json` defines the useful scripts:
  - `dev:full` builds wasm first, then starts Vite
  - `dev` starts Vite only
  - `build` runs TypeScript typechecking before the production build
- `vite.config.ts` uses `orxParallelWasm(...)` to build the wasm package, configure the persistent worker, and set the required isolation headers
- `vite.config.ts` uses `worker.format: "es"` so the module worker is emitted in ESM form
- `tsconfig.json` uses `moduleResolution: "Bundler"` and `types: ["vite/client"]` so the generated wasm package and Vite imports typecheck cleanly

Importantly, parallel computation will not work without `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers. As mentioned above, this is added to Vite configuration. If you serve the built `dist/` folder outside Vite, the server must send the same COOP/COEP headers. A plain static server like `npx serve dist` will not work for threaded wasm because the browser will reject `SharedArrayBuffer` unless `self.crossOriginIsolated` is true. This repo includes `npm run serve:dist`, which serves `dist/` locally with the required headers.

## minimal flow to run locally

Vite:

```bash
npm run dev

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
