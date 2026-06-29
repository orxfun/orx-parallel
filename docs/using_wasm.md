# Using orx-parallel in WebAssembly

This guide explains how to take a compute-heavy Rust algorithm that already uses `orx-parallel` and make it available in browser wasm builds.

If you want the low-level build/runtime matrix and troubleshooting details first, also see `docs/wasm_web_threads.md`.

## Who this is for

You have:

- a parallel rust algorithm using `orx-parallel`,
- a goal to run it in the browser via wasm,
- a frontend (plain TS, Vite, React, etc.) that calls wasm exports.

## Step 1: Start with a two-folder project layout

Use a structure that separates Rust compute code from web app code:

```text
my-wasm-project/
    crate/
        Cargo.toml
        src/
            lib.rs
            computation.rs
    web/
        package.json
        src/
            main.ts
```

Recommended responsibilities:

- `crate/`: Rust wasm library exposing a minimal `#[wasm_bindgen]` API boundary.
- `crate/src/lib.rs`: wasm boundary layer where `#[wasm_bindgen]` exports are defined.
- `crate/src/computation.rs`: pure Rust computation module without wasm-specific dependencies. This file is optional; the computation can instead live in a separate dependency crate.
- `web/`: frontend app that loads wasm, initializes runtime, and calls exported functions.

This mirrors all example demos in this repository and keeps responsibilities clean.

Path mapping used in the remaining steps:

- Step 2: `my-wasm-project/crate/Cargo.toml`
- Step 3: `my-wasm-project/crate/src/lib.rs`
- Step 4: `my-wasm-project/crate/src/computation.rs` (or a separate dependency crate)
- Step 5: `my-wasm-project/web/src/main.ts`
- Step 7: `my-wasm-project/web/package.json`
- Step 8: `my-wasm-project/web/vite.config.ts`

## Step 2: Add the right crate dependencies

In `my-wasm-project/crate/Cargo.toml`, use `orx-parallel` with wasm threads and add wasm bindings:

This demo targets at least version `4.0.0` of `orx-parallel`.

```toml
[dependencies]
orx-parallel = { version = "4.0.0", features = ["wasm-web-threads"] }
wasm-bindgen = "0.2"
js-sys = "0.3"
serde = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

For examples, see:

- `examples/wasm_demo_tsp/crate/Cargo.toml`
- `examples/wasm_vite_demo_basic/crate/Cargo.toml`
- `examples/wasm_react_demo_basic/crate/Cargo.toml`

## Step 3: Expose a wasm API boundary

Keep wasm-bindgen on a small public boundary layer in `my-wasm-project/crate/src/lib.rs`.

Expose (i) one runtime initialization function and (ii) entry points for the computations.

### Runtime initialization function

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


### Entry points for computations

Notice that this is just an entry point; the actual computation is implemented in the `computation` module which intentionally avoids wasm dependencies.

```rust
#[wasm_bindgen]
pub fn run_best_tour_par(
    iterations: u32,
    seed: u64,
    threads: u32,
    num_cities: u32,
    start_index: u64,
) -> Result<JsValue, JsValue> {
    #[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
    {
        let _ = (iterations, seed, threads, num_cities, start_index);
        return Err(JsValue::from_str(
            "run_best_tour_par requires wasm32 + atomics build",
        ));
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
    {
        let iterations = iterations.max(1) as usize;
        let threads = threads.max(1) as usize;
        let num_cities = locations::clamp_num_cities(num_cities);
        let output =
            computation::run_search_parallel(iterations, seed, threads, num_cities, start_index);
        run_output_to_js(output)
    }
}
```

### Why to use `cfg` attributes:

- threaded wasm builds get a Promise-based init path,
- unsupported builds fail fast with a clear error.

For reference:

- `examples/wasm_demo_tsp/crate/src/lib.rs`
- `examples/wasm_vite_demo_basic/crate/src/lib.rs`
- `examples/wasm_react_demo_basic/crate/src/lib.rs`

## Step 4: Keep computation modules pure Rust

Place heavy compute logic in internal modules and avoid wasm-specific code there. These functions can be implemented in `my-wasm-project/crate/src/computation.rs` for instance, or a separate dependency crate.

This keeps computation code testable and reusable.

```rust
/// Runs a parallel TSP search chunk and returns algorithm output.
pub fn run_search_parallel(
    iterations: usize,
    seed: u64,
    threads: usize,
    num_cities: usize,
    start_index: u64,
) -> SearchRunOutput {
    let best = (0..iterations)
        .into_par()
        .num_threads(threads)
        .map(|k| search_candidate(seed, start_index.wrapping_add(k as u64), num_cities))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Equal));

    SearchRunOutput { best, iterations }
}
```

Keep time-keeping in the caller (wasm boundary) when you need wall-clock timing in response payloads.

Example mapping for your project:

- wasm boundary: `my-wasm-project/crate/src/lib.rs`
- computation module: `my-wasm-project/crate/src/computation.rs` (or a dependency crate)

## Step 5: Initialize runtime once in frontend startup

After loading wasm module, call `init_parallel_runtime(...)` once and await it before parallel runs.

Typical frontend flow:

1. `await init()`
2. `await init_parallel_runtime(...)`
3. enable parallel actions

In your project, this is typically done in `my-wasm-project/web/src/main.ts`.

## Step 6: Decide thread-control strategy

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

- `examples/wasm_demo_tsp/crate/src/computation.rs`
- `examples/wasm_demo_tsp/web/src/main.ts`

## Step 7: Build with wasm thread flags

Use nightly + `build-std` + atomics/shared-memory flags.

If you run this from `my-wasm-project/web`, use:

```bash
RUSTUP_TOOLCHAIN=nightly \
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 -C link-arg=--import-memory -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base' \
wasm-pack build ../crate --target web --out-dir ../web/pkg -- -Z build-std=panic_abort,std
```

See:

- `examples/wasm_demo_tsp/web/package.json`
- `examples/wasm_vite_demo_basic/web/package.json`
- `examples/wasm_react_demo_basic/web/package.json`

## Step 8: Serve with COOP/COEP headers

Browser wasm threads require cross-origin isolation.

Set headers in dev server:

- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`

In your project, place these headers in `my-wasm-project/web/vite.config.ts`.

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

4. Re-initializing runtime repeatedly.

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
