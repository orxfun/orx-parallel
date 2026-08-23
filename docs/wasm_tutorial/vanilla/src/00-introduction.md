# Introduction

This tutorial demonstrates how to run computations expressed with `orx-parallel`'s parallel iterators in a threaded WebAssembly browser app.

The project has three parts:

1. `computation/` contains ordinary Rust code and tests.
2. `wasm_bindings/` exposes the Rust functions to JavaScript and initializes the parallel runtime.
3. `app/` contains the HTML, CSS, TypeScript, and Vite configuration. The TypeScript client calls WASM through a worker.

The app starts one shared thread pool. `threads: 0` lets the runtime choose the available capacity. Each computation also receives a thread count: `0` uses all initialized threads, while a positive value limits that computation.

## Prerequisites

Install Rust and Cargo, Node.js and npm, and the `wasm32-unknown-unknown` Rust target:

```bash
rustup target add wasm32-unknown-unknown
```

Create your example application's directory

```bash
mkdir par_wasm
cd par_wasm
```

The next chapter creates the first crate. Run later commands from the repository root unless a chapter says to change directories.
