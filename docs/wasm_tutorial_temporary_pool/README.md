# Wasm Vanilla Tutorial

This tutorial shows how to build a minimal JavaScript frontend for `orx-parallel` in wasm.

## Build as HTML with mdBook

An mdBook project is now wired in this folder:

- config: `book.toml`
- source: `src/`
- generated html: `book/`

Install mdBook if needed:

```bash
cargo install mdbook
```

Build the book:

```bash
cd docs/wasm_tutorial_temporary_pool
mdbook build
```

Serve locally with live reload:

```bash
cd docs/wasm_tutorial_temporary_pool
mdbook serve --open
```
