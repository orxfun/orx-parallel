# 04 - Build and Run

[Previous: 03 - Frontend App](03-frontend-app.md) | [Next: Tutorial Home](index.md)

Create `app/package.json`:

```json
{
    "name": "orx-parallel-wasm-fib-vanilla-app",
    "private": true,
    "type": "module",
    "scripts": {
        "build:wasm": "RUSTUP_TOOLCHAIN=nightly RUSTFLAGS='-C target-feature=+atomics -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 -C link-arg=--import-memory -C link-arg=--export=__heap_base -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base' wasm-pack build ../wasm_bindings --target web --out-dir ../app/pkg -- -Z build-std=panic_abort,std && find ../app/pkg/snippets -path '*/src/pool/pool_impl/wasm_web_start_workers.js' -exec sh -c 'cp $1 ${1%wasm_web_start_workers.js}worker_helpers.js' _ {} \\;",
        "dev:full": "npm run build:wasm && npm exec -- vite",
        "dev": "npm exec -- vite"
    },
    "devDependencies": {
        "vite": "^5.4.10"
    }
}
```

Create `app/vite.config.js`:

```js
import { defineConfig } from "vite";

export default defineConfig({
    base: "./",
    server: {
        headers: {
            "Cross-Origin-Opener-Policy": "same-origin",
            "Cross-Origin-Embedder-Policy": "require-corp"
        }
    },
    worker: {
        format: "es"
    }
});
```

**>_** Install and run:

```bash
cd app
npm install
npm run dev:full
```

**>_** Checkpoint: 

Open the Vite URL and run a computation:

* Enter `1` for number of threads, press `run` and wait until the result `2601945768813055516` is displayed. This runs the computation on a single thread.

* Enter `0` to use all available threads, press `run` and wait until the result `2601945768813055516` is displayed.

* Now enter `4`, `16` and `32` for number of threads, `run` and wait until you see the same result `2601945768813055516`.

## Common pitfalls

- Missing COOP/COEP headers: browser threads will not work.
- Skipping `init_wasm_parallel_runtime`: parallel path will not initialize.
- Reusing stale `pkg`: rerun `npm run build:wasm` after Rust changes.
