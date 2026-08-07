# 03 - Frontend App

[Previous: 02 - Wasm Bindings Crate](02-wasm-bindings-crate.md) | [Next: 04 - Build and Run](04-build-and-run.md)

Create `app/index.html`:

```html
<!doctype html>
<html>
  <body>
    <label>
      input
      <input id="input" type="number" value="100000" min="1" />
    </label>

    <label>
      threads
      <input id="threads" type="number" value="4" min="1" />
    </label>

    <button id="run">run</button>

    <p id="result">-</p>

    <script type="module" src="./src/main.js"></script>
  </body>
</html>
```

Create `app/src/main.js`:

```js
const inputEl = document.getElementById("input");
const threadsEl = document.getElementById("threads");
const runEl = document.getElementById("run");
const resultEl = document.getElementById("result");

runEl.addEventListener("click", () => {
  const input = Math.max(1, Number(inputEl.value) || 1);
  const threads = Math.max(1, Number(threadsEl.value) || 1);

  const worker = new Worker(new URL("./worker.js", import.meta.url), {
    type: "module"
  });

  worker.addEventListener(
    "message",
    (event) => {
      const data = event.data;

      if (data.type === "ok") {
        resultEl.textContent = String(data.result);
      } else {
        resultEl.textContent = `error: ${data.message}`;
      }

      worker.terminate();
    },
    { once: true }
  );

  worker.postMessage({ input, threads });
});
```

Create `app/src/worker.js`:

```js
import init, { init_parallel_runtime, run } from "../pkg/wasm_bindings.js";

self.addEventListener("message", async (event) => {
  try {
    const { input, threads } = event.data;

    await init();
    await init_parallel_runtime(threads);

    const result = run(input, threads);
    self.postMessage({ type: "ok", result });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    self.postMessage({ type: "err", message });
  }
});
```

This fulfills the 4-element UI requirement exactly.
