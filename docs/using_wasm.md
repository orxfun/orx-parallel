# Using orx-parallel in WebAssembly

This guide explains how to take a compute-heavy Rust algorithm that already uses `orx-parallel` and make it available in browser wasm builds.

If you want the low-level build/runtime matrix and troubleshooting details first, also see `docs/wasm_web_threads.md`.

## Who this is for

You have:

- a Rust algorithm using `orx-parallel`,
- a goal to run it in the browser via wasm,
- a frontend (plain TS, Vite, React, etc.) that calls wasm exports.

## Step 1: Add the right crate dependencies

In your wasm crate `Cargo.toml`, use `orx-parallel` with wasm threads and add wasm bindings:

```toml
[dependencies]
orx-parallel = { path = "../../..", features = ["wasm-web-threads"] }
wasm-bindgen = "0.2"
js-sys = "0.3"
serde = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

For examples, see:

- `examples/wasm_demo_tsp/crate/Cargo.toml`
- `examples/wasm_vite_demo_basic/crate/Cargo.toml`
- `examples/wasm_react_demo_basic/crate/Cargo.toml`

## Step 2: Expose a wasm API boundary

Keep wasm-bindgen on a small public boundary layer (typically `lib.rs`).

Expose at least:

1. one runtime initialization function,
2. one or more compute entry points.

Example shape:

```rust
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[wasm_bindgen]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    orx_parallel::init_thread_pool(num_threads as usize)
}

#[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
#[wasm_bindgen]
pub fn init_parallel_runtime(_num_threads: u32) -> Result<JsValue, JsValue> {
    Err(JsValue::from_str(
        "init_parallel_runtime is only available for wasm32 + atomics builds",
    ))
}
```

Why this split matters:

- threaded wasm builds get a Promise-based init path,
- unsupported builds fail fast with a clear error.

For reference:

- `examples/wasm_demo_tsp/crate/src/lib.rs`
- `examples/wasm_vite_demo_basic/crate/src/lib.rs`
- `examples/wasm_react_demo_basic/crate/src/lib.rs`

## Step 3: Keep algorithm modules pure Rust when possible

Place heavy compute logic in internal modules and avoid wasm-specific code there unless timing/interop requires it.

This keeps algorithm code testable and reusable.

Example:

- wasm boundary: `examples/wasm_demo_tsp/crate/src/lib.rs`
- algorithm module: `examples/wasm_demo_tsp/crate/src/tsp_alg.rs`

## Step 4: Initialize runtime once in frontend startup

After loading wasm module, call `init_parallel_runtime(...)` once and await it before parallel runs.

Typical frontend flow:

1. `await init()`
2. `await init_parallel_runtime(...)`
3. enable parallel actions

Example from TSP demo startup:

- `examples/wasm_demo_tsp/web/src/main.ts`

## Step 5: Decide thread-control strategy

There are two good patterns.

### Pattern A: Fixed runtime + fixed computation threads

Good for simple demos.

- initialize runtime with fixed thread count,
- run parallel operations with defaults.

See:

- `examples/wasm_vite_demo_basic`
- `examples/wasm_react_demo_basic`

### Pattern B: Fixed startup cap + per-run limit

Good for interactive apps.

- initialize runtime once with a cap (for example `16`),
- per computation use `.num_threads(threads)` in Rust pipeline.

Example:

```rust
let best = (0..iterations)
    .into_par()
    .num_threads(threads)
    .map(...)
    .min_by(...);
```

See:

- `examples/wasm_demo_tsp/crate/src/tsp_alg.rs`
- `examples/wasm_demo_tsp/web/src/main.ts`

## Step 6: Build with wasm thread flags

Use nightly + `build-std` + atomics/shared-memory flags.

Example script (from demos):

```bash
RUSTUP_TOOLCHAIN=nightly \
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 -C link-arg=--import-memory -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base' \
wasm-pack build ../crate --target web --out-dir ../web/pkg -- -Z build-std=panic_abort,std
```

See:

- `examples/wasm_demo_tsp/web/package.json`
- `examples/wasm_vite_demo_basic/web/package.json`
- `examples/wasm_react_demo_basic/web/package.json`

## Step 7: Serve with COOP/COEP headers

Browser wasm threads require cross-origin isolation.

Set headers in dev server:

- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`

See Vite config examples:

- `examples/wasm_demo_tsp/web/vite.config.ts`
- `examples/wasm_vite_demo_basic/web/vite.config.ts`
- `examples/wasm_react_demo_basic/web/vite.config.ts`

## Common mistakes

1. Calling parallel code before `await init_parallel_runtime(...)`.

- Symptom: runtime panic/trap in browser.

2. Missing atomics/shared-memory build flags.

- Symptom: threaded wasm path fails at runtime or behaves unexpectedly.

3. Assuming every Rust method appears in JS API.

- Only `#[wasm_bindgen]` exports are visible to JS.
- Internal iterator methods like `.num_threads(...)` are Rust-side API.

4. Treating HTML input `min`/`max` as strict validation.

- Enforce bounds in code too (parse + clamp) for reliability.

5. Re-initializing runtime repeatedly.

- Usually initialize once at startup; vary per-run behavior using computation parameters.

## Minimal checklist

- [ ] Add `wasm-web-threads` feature and wasm dependencies.
- [ ] Export `init_parallel_runtime(...)` and compute entry points via `#[wasm_bindgen]`.
- [ ] Await runtime init once before parallel runs.
- [ ] Configure wasm thread build flags.
- [ ] Configure COOP/COEP headers in dev/prod serving.
- [ ] Validate with `wasm-pack build` and frontend type-check/build.

## Example demos

- Advanced interactive example: `examples/wasm_demo_tsp`
- Basic plain TS example: `examples/wasm_vite_demo_basic`
- Basic React example: `examples/wasm_react_demo_basic`
