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
