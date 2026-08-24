# Rollup variant

This is the same mini demo as `app`, built with Rollup instead of Vite.

`rollup.config.mjs` uses `orx-parallel-wasm/rollup`, which builds the Rust
crate with `wasm-pack`, fixes up the generated worker snippets, and emits the
prepared package into `dist/assets` alongside a `_headers` file — the same
layout produced by the Vite and Webpack plugins. The application always
resolves the bindings from the stable `assets/bindings.js` entry the plugin
emits, so it does not need to know the crate's actual output filename.

Rollup has no built-in HTML/CSS pipeline or dev server, so `rollup.config.mjs`
also copies `index.html`/`style.css` verbatim and uses `rollup-plugin-serve`
(configured with the cross-origin isolation headers required by
`SharedArrayBuffer`) for `npm run dev`. A production server or host must set
the same headers when serving `dist` (see the generated `_headers` file).

Note that no live-reload plugin is used. `rollup-plugin-livereload` serves its
client script from a separate origin (`localhost:35729`), and COEP
`require-corp` blocks cross-origin subresources that do not send a CORP header.
`rollup -c --watch` still rebuilds on every change; refresh the browser to see
them.

```bash
npm install
npm run dev
```

For a production build:

```bash
npm run build
```

