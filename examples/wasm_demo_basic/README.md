# orx-parallel wasm demo basic

A minimal wasm demo that computes a Fibonacci sum in parallel with `orx-parallel`.

## What it shows

- Rust -> wasm compute module
- Browser thread-pool initialization
- Parallel computation with desired number of threads
- Plain HTML + TypeScript frontend (no custom styling)

## Run the demo

```bash
cd examples/wasm_demo_basic/web
npm install
npm run dev:full
```

Open the printed local URL and click **Run parallel Fibonacci sum**.

## Rebuild wasm after Rust changes

```bash
cd examples/wasm_demo_basic/web
npm run build:wasm
```

Then refresh the browser.
