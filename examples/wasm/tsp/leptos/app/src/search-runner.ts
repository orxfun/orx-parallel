import type { SearchRequest, SearchResult, SearchResponse } from "./shared-types.js";

export function runSearchOnce(request: SearchRequest): Promise<SearchResult> {
    return new Promise<SearchResult>((resolve, reject) => {
        const worker = new Worker(new URL("./search-worker.ts", import.meta.url), {
            type: "module"
        });

        const cleanup = () => {
            worker.terminate();
        };

        worker.addEventListener(
            "message",
            (event: MessageEvent) => {
                const data = event.data as SearchResponse;

                if (data.type === "search-error") {
                    cleanup();
                    reject(new Error(data.message));
                    return;
                }

                cleanup();
                resolve(data.result);
            },
            { once: true }
        );

        worker.addEventListener(
            "error",
            (event) => {
                cleanup();
                reject(new Error(event.message || "search worker failed"));
            },
            { once: true }
        );

        worker.postMessage(request);
    });
}
// Expose the function globally so wasm-bindgen can call it from Rust via globalThis.runSearchOnce.
(globalThis as typeof globalThis & { runSearchOnce: typeof runSearchOnce }).runSearchOnce = runSearchOnce;

export { };
