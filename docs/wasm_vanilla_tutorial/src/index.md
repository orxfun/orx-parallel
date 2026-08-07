# Parallellizaton in Vanilla JS UI with `orx-parallel`

This tutorial shows how to build a minimal JavaScript frontend to achieve parallel computaiton using `orx-parallel` in wasm.

The UI is minimalistic for demonstration purposes:

1. It has an integer input which determines the **input** to the computaiton.
2. It has another integer input that configures the **number of threads** to be used.
3. One button to start the computation.
4. One text **output** to display the result.

The user enters the **input**, **number of threads** and presses the button. The computation is parallelized using `orx-parallel` and the result is displayed in the **output**.

## Chapters

1. [00 - Introduction](00-introduction.md)
2. [01 - Computation Crate](01-computation-crate.md)
3. [02 - Wasm Bindings Crate](02-wasm-bindings-crate.md)
4. [03 - Frontend App](03-frontend-app.md)
5. [04 - Build and Run](04-build-and-run.md)
