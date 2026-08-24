# Webpack variant

This is the same mini demo as `app`, built with Webpack instead of Vite.

`webpack.config.js` uses `orx-parallel-wasm/webpack`, which builds the Rust
crate with `wasm-pack`, fixes up the generated worker snippets, and emits the
prepared package into `dist/assets` alongside a `_headers` file — the same
layout produced by the Vite plugin. The application always resolves the
bindings from the stable `assets/bindings.js` entry the plugin emits, so it
does not need to know the crate's actual output filename.

The plugin also merges the cross-origin isolation headers required by
`SharedArrayBuffer` into `devServer`. A production server or host must set the
same headers when serving `dist` (see the generated `_headers` file).

```bash
npm install
npm run dev
```

For a production build:

```bash
npm run build
```
