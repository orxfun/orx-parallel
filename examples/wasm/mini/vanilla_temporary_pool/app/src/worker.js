import init, { init_wasm_parallel_runtime, compute } from "../pkg/wasm_bindings.js";

self.addEventListener("message", async (event) => {
    try {
        const { input, threads } = event.data;

        await init();
        await init_wasm_parallel_runtime(threads);

        const result = compute(input, threads);
        self.postMessage({ type: "ok", result });
    } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        self.postMessage({ type: "err", message });
    }
});
