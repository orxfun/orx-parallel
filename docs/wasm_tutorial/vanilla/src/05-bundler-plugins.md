# Other bundlers

The app built in this tutorial uses Vite, but the WASM bindings can be used with other JavaScript bundlers. The `orx-parallel-wasm` package provides a bundler-specific plugin for each integration:

* [Vite](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/mini/vanilla-vite)
* [Webpack](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/mini/vanilla-webpack)
* [Rspack](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/mini/vanilla-rspack)
* [Rollup](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/mini/vanilla-rollup)

Each example is the same small application with a different bundler configuration. The plugin is imported from the corresponding subpath, such as `orx-parallel-wasm/vite` or `orx-parallel-wasm/webpack`.

## Why the plugins exist

A Rust crate compiled for browser threads is more involved than importing a single `.wasm` file. The plugin coordinates the build and packaging steps that connect Rust, WebAssembly, JavaScript, and the bundler:

* it invokes the WASM build for the bindings crate with the required `wasm32-unknown-unknown`, atomics, and shared-memory settings;
* it prepares the `wasm-bindgen` JavaScript and worker code so the browser can create the parallel worker correctly;
* it emits the generated WASM and JavaScript as normal bundler assets, including a stable bindings entry for the application; and
* it provides or records the `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers required by `SharedArrayBuffer`.

Without a plugin, these steps would need to be kept in sync with the bundler's hooks, asset graph, output directory, worker handling, and development server. The details differ between bundlers, which is why `orx-parallel-wasm` exposes separate integrations rather than one generic configuration.

Rollup is intentionally more low-level than Vite, Webpack, or Rspack. Its example also copies the HTML and CSS files and configures a development server, because Rollup does not provide those pieces by itself.

## Building it yourself

It is possible to build the application without one of these plugins. The essential workflow is:

1. Run the WASM build as a separate step, targeting `wasm32-unknown-unknown` with atomics and shared memory enabled (see the command in [The WASM bindings crate](02-wasm-bindings-crate.html#build-optional)).
2. Choose a known public or bundled location for the generated bindings, WASM module, and worker assets.
3. Configure the bundler to copy or emit those assets without changing the URLs expected by the generated worker code.
4. Initialize `ParallelWorker` with the URL of the generated bindings entry.
5. Configure the development server to send `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp`.
6. Configure the production server or hosting platform to send the same headers when serving the built application.

A complete plugin-free version of this demo is available in the [vanilla-manual example](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/mini/vanilla-manual). It uses the bundler-neutral build command, `esbuild` only to bundle the application TypeScript, and a small Node.js server to serve the generated assets with the required headers.

This approach gives more control over output names, caching, deployment, and bundler behavior, but it also makes the integration the application's responsibility. The examples above are useful references when implementing that workflow manually or when adapting the plugin to a different build system.
