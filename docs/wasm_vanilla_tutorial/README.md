# Wasm Vanilla Tutorial

This tutorial shows how to build a minimal JavaScript frontend for `orx-parallel` in wasm.

Target UI has only 4 elements:

1. integer input for `input`
2. integer input for number of threads
3. one button to start computation
4. one text output for the result

No extra Rust logic, no CSS styling, no extra HTML elements.

## Chapters

1. [00 - Introduction](00-introduction.md)
2. [01 - Computation Crate](01-computation-crate.md)
3. [02 - Wasm Bindings Crate](02-wasm-bindings-crate.md)
4. [03 - Frontend App](03-frontend-app.md)
5. [04 - Build and Run](04-build-and-run.md)

## Output format decision

Use markdown chapter files in this repository.

Reasoning:

- Highest maintainability in a code repo (diff-friendly, reviewable, easy to version).
- Same source can later be rendered as a Rust-book-style HTML site via mdBook if desired.
- Keeps docs close to code and examples with minimal tooling friction.
