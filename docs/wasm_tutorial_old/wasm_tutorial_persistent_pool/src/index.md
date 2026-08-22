# Parallellizaton in Vanilla JS UI with `orx-parallel` and temporary worker

This tutorial shows how to build a minimal JavaScript frontend to achieve parallel computaiton using `orx-parallel` in wasm.

The UI is minimalistic for demonstration purposes:

1. It has an integer input which determines the **input** to the computaiton.
2. It has another integer input that configures the **number of threads** to be used.
3. One button to start the computation.
4. One text **output** to display the result.

The user enters the **input**, **number of threads** and presses the button. The computation is parallelized using `orx-parallel` and the result is displayed in the **output**.

## Temporary vs Persistent Worker

In this demo, a persistent worker and thread pool are used. This might reduce overhead.

The alternative is to create a worker and a thread pool per computation, which is a clean and flexible approach.

These two approaches differ only in the `app` and either one can be taken depending on the application.

You may access the source code created by following the tutorials:
* in [`examples/wasm/mini/vanilla_temporary_pool`](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/mini/vanilla_temporary_pool) creating a worker per computation, and
* in [`examples/wasm/mini/vanilla_persistent_pool`](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/mini/vanilla_persistent_pool) using a single persistent worker.


## Chapters

1. [00 - Introduction](00-introduction.md)
2. [01 - Computation Crate](01-computation-crate.md)
3. [02 - Wasm Bindings Crate](02-wasm-bindings-crate.md)
4. [03 - Frontend App](03-frontend-app.md)
5. [04 - Build and Run](04-build-and-run.md)
