# Vanilla `orx-parallel` WASM tutorial

This tutorial builds a small TypeScript browser app that runs two Rust computations through `orx-parallel-wasm`:

- a Fibonacci workload
- a Mandelbrot checksum

The finished example is in [`examples/wasm/mini/vanilla`](../../../examples/wasm/mini/vanilla), but you can create it by following the tutorial without opening that directory.

The chapters create the project in this order:

1. Create the Rust computation crate.
2. Add the WASM bindings crate.
3. Create the Vite app and its worker-backed TypeScript UI.
4. Build and run the app.

To build and run it:

```text
cd examples/wasm/mini/vanilla/app
npm install
npm run build
npm run dev
```

Open the URL printed by Vite. The dev server supplies the cross-origin isolation headers required by threaded WebAssembly.
