function waitForMsgType(target, type) {
    return new Promise((resolve) => {
        target.addEventListener("message", function onMsg({ data }) {
            if (data == null || data.type !== type) return;
            target.removeEventListener("message", onMsg);
            resolve(data);
        });
    });
}

waitForMsgType(self, "orx_parallel_worker_init").then(async ({ init }) => {
    try {
        const pkg = await import("../../../../..");
        await pkg.default(init);
        postMessage({ type: "orx_parallel_worker_ready" });
        if (typeof pkg.wasm_web_start_worker !== "function") {
            throw new Error("wasm worker entrypoint is missing: expected wasm_web_start_worker");
        }
        pkg.wasm_web_start_worker();
    } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        postMessage({ type: "orx_parallel_worker_error", message });
    }
});

let _workers;

export async function startWorkers(module, memory, numThreads) {
    if (numThreads === 0) {
        throw new Error("num_threads must be > 0.");
    }

    const workerInit = {
        type: "orx_parallel_worker_init",
        init: { module_or_path: module, memory },
    };

    _workers = await Promise.all(
        Array.from({ length: numThreads }, async (_, idx) => {
            const worker = new Worker(new URL("./worker_helpers.js", import.meta.url), {
                type: "module",
            });

            const onWorkerError = new Promise((_, reject) => {
                worker.addEventListener("error", (event) => {
                    reject(new Error(`worker ${idx} script error: ${event.message || "unknown error"}`));
                }, { once: true });
            });

            const onWorkerInitError = waitForMsgType(worker, "orx_parallel_worker_error").then((data) => {
                throw new Error(`worker ${idx} init failed: ${data.message || "unknown error"}`);
            });

            const onWorkerReady = waitForMsgType(worker, "orx_parallel_worker_ready");

            const onWorkerTimeout = new Promise((_, reject) => {
                setTimeout(() => reject(new Error(`worker ${idx} init timed out`)), 10000);
            });

            worker.postMessage(workerInit);

            await Promise.race([onWorkerReady, onWorkerInitError, onWorkerError, onWorkerTimeout]);
            return worker;
        })
    );
}
