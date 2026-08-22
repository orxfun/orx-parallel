# Vanilla mini WASM example

This is the runnable project for the vanilla tutorial in `docs/wasm_tutorial/vanilla`.

```text
cd app
npm install
npm run build
npm run dev
```

The app uses `orx-parallel-wasm` and a worker-backed TypeScript UI. The Rust computation and bindings crates are siblings of `app`.
