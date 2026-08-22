# Build and run

From the app directory:

```text
npm install
npm run typecheck
npm run build
npm run dev
```

`npm run build` first runs the package build script. The Vite plugin compiles `wasm_bindings/`, enables threaded WASM, and writes the generated module to `app/pkg`. Vite then typechecks and bundles the TypeScript app.

The development server sends these headers:

```text
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

They are required for `SharedArrayBuffer` and browser threads. Use the thread input to compare `0` (all initialized threads) with a smaller positive count. Increase either workload if the displayed duration is too short to compare.
