# Other frameworks

This tutorial uses vanilla JavaScript and TypeScript to keep the browser-facing code as small and transparent as possible. The computation crate, WASM bindings crate, worker boundary, and browser requirements do not depend on that choice of UI framework. React can use the same structure with components and state managing the page instead of direct DOM updates.

## React with Vite

The [React + Vite mini example](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/mini/react-vite) provides the same Fibonacci and Mandelbrot demo using React. Its `computation/` and `wasm_bindings/` crates are identical to the ones used by the other `examples/wasm/mini` projects, and its stylesheet and output match the vanilla example.

The React entrypoint creates a `ParallelWorker` client in the same way as the vanilla app, giving it the generated bindings URL, exported method names, and desired thread count. `ParallelWorker` creates the module worker internally, and the client instance is passed to the `App` component as a prop. `App` uses React state for input values, status messages, results, and button state, while calls still cross the same worker and `ParallelWorker` boundary.

This separation is useful in a larger application: React owns rendering and UI state, while `ParallelWorker` owns communication with the module worker and the WASM bindings. The computation itself remains in Rust.

## Rust UI frameworks

The [TSP examples](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/tsp) include additional applications built with different frontend approaches:

* [Vanilla TypeScript](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/tsp/vanilla) uses Vite and direct DOM updates.
* [React](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/tsp/react) uses React components with a Vite host application.
* [Yew](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/tsp/yew) uses a Rust Yew component crate hosted by a Vite browser application.
* [Leptos](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/tsp/leptos) uses a Rust Leptos component crate with a Vite browser host.

The Vanilla and React examples keep the UI in JavaScript or TypeScript. The Yew and Leptos examples move the UI into Rust and compile it to WASM, but the architecture is still recognizable: a computation crate contains the algorithm, a bindings boundary exposes the WASM API, and the browser application owns initialization, worker lifecycle, and cross-origin isolation.

For the published demos, see the [TSP example hub](https://orx-parallel-wasm-demo-tsp.pages.dev/). These examples are larger than the mini tutorial, but they demonstrate that the same parallel WASM design can be adapted to vanilla JavaScript, React, Yew, or Leptos without changing the core computation model.
