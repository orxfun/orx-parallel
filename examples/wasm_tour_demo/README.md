# orx-parallel wasm tour demo

This demo shows a complete setup for running `orx-parallel` in the browser with wasm threads.

## What this demo contains

- `crate/`: Rust compute module compiled to wasm.
- `web/`: Vite + TypeScript UI that loads wasm, initializes the thread pool, and runs search jobs.

## Step-by-step setup

1. Install prerequisites

```bash
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo install wasm-pack
```

2. Install frontend dependencies

```bash
cd examples/wasm_tour_demo/web
npm install
```

3. Build the Rust wasm package

```bash
cd examples/wasm_tour_demo/web
npm run build:wasm
```

4. Start the frontend

```bash
cd examples/wasm_tour_demo/web
npm run dev
```

5. Open the printed URL (usually `http://localhost:5173`), then:

- choose iterations, threads, seed, and number of cities
- click `Run parallel search` or `Run sequential search`

## Fast path commands

- Build wasm and run dev server in one step:

```bash
cd examples/wasm_tour_demo/web
npm run dev:full
```

## Rebuild workflow while developing

1. Edit Rust in `crate/src`.
2. Rebuild wasm:

```bash
cd examples/wasm_tour_demo/web
npm run build:wasm
```

3. Refresh the browser page.

## How the frontend flow works

The frontend logic in `web/src/main.ts` is organized as:

1. `setupApp()`: initializes wasm, loads city points, and wires all UI handlers.
2. `runSearch(mode)`: reads settings, initializes threads (parallel mode), then runs chunked searches.
3. `runSearchChunk(...)`: calls wasm exports for either parallel or sequential chunk execution.
4. Overlay helpers (`setRunningView`, `allowRunningOverlayToRender`): show progress and support cancellation.
5. Canvas helpers (`drawPoints`, `drawTour`, `mapPoints`): render cities and best tour.

## Important runtime notes

- `init_thread_pool(...)` is required before first parallel run.
- `Cancel Run` is cooperative: it stops after the current chunk.
- Vite dev server is configured with COOP/COEP headers required for SharedArrayBuffer + wasm threads.
