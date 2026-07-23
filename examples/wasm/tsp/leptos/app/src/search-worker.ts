import init, { init_parallel_runtime, run_search } from "../pkg/components.js";
import type { SearchRequest, SearchResult, SearchResponse } from "./shared-types.js";

self.addEventListener("message", async (event: MessageEvent<SearchRequest>) => {
    try {
        const settings = event.data.settings;
        await init();

        if (settings.mode === "parallel") {
            await init_parallel_runtime(settings.threads);
        }

        const seed = normalizeSeed(settings.seed);
        const parallelize = settings.mode === "parallel";
        const result = run_search(
            parallelize,
            settings.iterations,
            seed,
            settings.threads,
            settings.chunkSize,
            event.data.locations
        ) as SearchResult;

        const response: SearchResponse = { type: "search-result", result };
        self.postMessage(response);
    } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        const response: SearchResponse = {
            type: "search-error",
            message
        };
        self.postMessage(response);
    }
});

function normalizeSeed(seed: number | bigint | string): bigint {
    if (typeof seed === "bigint") {
        return seed;
    }

    if (typeof seed === "number") {
        if (!Number.isFinite(seed)) {
            throw new Error("invalid seed: expected a finite number");
        }

        return BigInt(Math.trunc(seed));
    }

    return BigInt(seed);
}