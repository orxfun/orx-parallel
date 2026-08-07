import init, { init_parallel_runtime, compute } from "../pkg/wasm_bindings.js";

let threads = 0;

self.addEventListener("message", async (event) => {
    try {
        const data = event.data;

        if (data.type === "init") {
            threads = data.threads;
            await init();
            await init_parallel_runtime(threads);
        } else if (data.type === "compute") {
            const result = compute(data.input, threads);
            self.postMessage({ type: "ok", result });
        }
    } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        self.postMessage({ type: "err", message });
    }
});
