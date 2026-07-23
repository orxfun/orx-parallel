import { locations } from "../pkg/components.js";
import { normalizeSeed } from "./shared-types.js";
import type { RunSettings, SearchRequest, SearchResult, SearchResponse } from "./shared-types.js";

export function runSearchOnce(settings: RunSettings): Promise<SearchResult> {
    return new Promise<SearchResult>((resolve, reject) => {
        const seed = normalizeSeed(settings.seed);
        const request: SearchRequest = {
            type: "run-search",
            settings,
            locations: locations(seed, settings.numCities) as { x: number; y: number }[]
        };

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
(globalThis as typeof globalThis & { runSearchOnce: typeof runSearchOnce }).runSearchOnce = runSearchOnce;

export { };
