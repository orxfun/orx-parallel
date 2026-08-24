# Rspack variant

This is the same mini demo as `app`, built with Rspack instead of Vite.

`rspack.config.mjs` uses `orx-parallel-wasm/rspack`, which builds the Rust
crate with `wasm-pack`, fixes up the generated worker snippets, and emits the
prepared package into `dist/assets` alongside a `_headers` file — the same
layout produced by the Vite, Webpack, and Rollup plugins. The application
always resolves the bindings from the stable `assets/bindings.js` entry the
plugin emits, so it does not need to know the crate's actual output filename.

`orx-parallel-wasm/rspack` re-exports the Webpack plugin unchanged: Rspack
implements the webpack plugin API (`compiler.webpack`, the same hook names,
and `emitAsset`), so no separate implementation is needed. The plugin merges
the cross-origin isolation headers required by `SharedArrayBuffer` into
`devServer`. A production server or host must set the same headers when
serving `dist` (see the generated `_headers` file).

```bash
npm install
npm run dev
```

For a production build:

```bash
npm run build
```

