# Build and run

From the app directory:

```text
npm install
npm run build
npm run dev
```

`npm run build` first compiles `wasm_bindings/` with the `orx-parallel-wasm` build script, writing the generated module to `app/pkg`. It then typechecks and bundles the TypeScript app with Vite. The Vite configuration points the plugin at the sibling `wasm_bindings/` crate, uses a thread pool size of `0` for automatic sizing, and emits a relative build suitable for the example.

The development server sends these headers:

```text
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

They are required for `SharedArrayBuffer` and browser threads. Use the thread input to compare `0` (all initialized threads) with a smaller positive count. Increase either workload if the displayed duration is too short to compare.
