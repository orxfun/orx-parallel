# 03 - Frontend App

[Previous: 02 - Wasm Bindings Crate](02-wasm-bindings-crate.md) | [Next: 04 - Build and Run](04-build-and-run.md)

**>_** Create the frontend directory `app, if not created

```bash
cd .. # go back to top level
mkdir app # if not yet created
```

**>_** Create the simple page `app/index.html`:

```html
<!doctype html>
<html>

<body>
    <label>
        input
        <input id="input" type="number" value="1000000" min="1" />
        the larger the input, the harder the computation
    </label>

    <br />

    <label>
        threads
        <input id="num_threads" type="number" value="1" min="0" max="32" />
        try with 1 for sequential; 0 to use all threads in the pool; 16 to use exactly 16 threads; etc.
    </label>

    <hr />

    <label>
        <button id="run">run</button>
        press & wait until the result is displayed below
    </label>

    <hr />

    <p id="result">-</p>

    <script type="module" src="./src/main.js"></script>
</body>

</html>
```

**>_** Create `app/src/main.js`:

```js
const inputEl = document.getElementById("input");
const threadsEl = document.getElementById("num_threads");
const runEl = document.getElementById("run");
const resultEl = document.getElementById("result");

// Set to `N > 0` to limit all computations by `worker` to at most `N` threads.
// Set to `0` to create the pool with all available threads.
const threadsInPool = 32;

// create one persistent worker
const worker = new Worker(new URL("./worker.js", import.meta.url), {
    type: "module"
});

// immediately initiate thread pool
worker.postMessage({ type: "init", threads: threadsInPool });

worker.addEventListener("message", (event) => {
    const data = event.data;

    if (data.type === "ok") {
        resultEl.textContent = String(data.result);
    } else if (data.type === "err") {
        resultEl.textContent = `error: ${data.message}`;
    }
});

runEl.addEventListener("click", () => {
    // this particular computation will use:
    // * `min(threadsInPool, threadsForComputation)` threads when `threadsForComputation > 0`,
    // * `threadsInPool` threads when `threadsForComputation == 0`.
    const threadsForComputation = Number(threadsEl.value);
    const input = Number(inputEl.value);
    resultEl.textContent = '-';
    worker.postMessage({ type: "compute", input, threadsForComputation });
});
```

Note that `main.js` is responsible for updating the UI state, creating the persistent worker, and to execute parallel computation when requested by the user.

**>_** Create `app/src/worker.js`:

```js
import init, { init_parallel_runtime, compute } from "../pkg/wasm_bindings.js";

self.addEventListener("message", async (event) => {
    try {
        const data = event.data;

        if (data.type === "init") {
            await init();
            await init_parallel_runtime(data.threadsInPool);
        } else if (data.type === "compute") {
            const result = compute(data.input, data.threadsForComputation);
            self.postMessage({ type: "ok", result });
        }
    } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        self.postMessage({ type: "err", message });
    }
});
```

`worker.js` initializes the parallel runtime or calls the `compute` function with the provided inputs.

***Notes***

* Note that we still need the next step to test parallel computation in the UI.
