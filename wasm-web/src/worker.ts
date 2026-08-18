import type { WasmBindings, WorkerRequest, WorkerResponse } from "./protocol.js";

let bindings: WasmBindings | undefined;
let allowedMethods = new Set<string>();
let initialization: Promise<void> | undefined;
let initializedThreads: number | undefined;

function respond(response: WorkerResponse): void {
    self.postMessage(response);
}

function initialize(request: Extract<WorkerRequest, { type: "init" }>): Promise<void> {
    if (initialization !== undefined) return initialization;

    initialization = (async () => {
        const imported = (await import(/* @vite-ignore */ request.bindingsUrl)) as WasmBindings;
        if (typeof imported.default !== "function") {
            throw new Error("wasm bindings must export a default initializer");
        }
        if (typeof imported.init_parallel_runtime !== "function") {
            throw new Error("wasm bindings must export init_parallel_runtime");
        }

        const threads = request.threads === 0
            ? Math.max(1, self.navigator?.hardwareConcurrency ?? 1)
            : request.threads;
        await imported.default();
        await imported.init_parallel_runtime(threads);
        bindings = imported;
        allowedMethods = new Set(request.methods);
        initializedThreads = threads;
    })();

    return initialization;
}

self.addEventListener("message", (event: MessageEvent<WorkerRequest>) => {
    const request = event.data;

    if (request.type === "init") {
        initialize(request)
            .then(() => respond({ type: "ready", threads: initializedThreads ?? request.threads }))
            .catch((error: unknown) => {
                const message = error instanceof Error ? error.message : String(error);
                respond({ type: "error", message });
            });
        return;
    }

    initializeCall(request).catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        respond({ type: "error", id: request.id, message });
    });
});

async function initializeCall(request: Extract<WorkerRequest, { type: "call" }>): Promise<void> {
    if (bindings === undefined) {
        throw new Error("worker is not initialized");
    }
    if (!allowedMethods.has(request.method)) {
        throw new Error(`method is not allowed: ${request.method}`);
    }

    const computation = bindings[request.method];
    if (typeof computation !== "function") {
        throw new Error(`wasm binding is not a function: ${request.method}`);
    }

    const value = await (computation as (...args: unknown[]) => unknown)(...request.args);
    respond({ type: "result", id: request.id, value });
}