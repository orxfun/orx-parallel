import init, { init_parallel_runtime, run_search } from "../pkg/wasm_bindings.js";

type SearchMode = "parallel" | "sequential";

type RunSettings = {
    mode: SearchMode;
    iterations: number;
    threads: number;
    chunkSize: number;
    seed: bigint;
    numCities: number;
};

type Location = {
    x: number;
    y: number;
};

type SearchResult = {
    best_tour: number[];
    best_distance: number;
    iterations: number;
    elapsed_ms: number;
};

type SearchRequest = {
    type: "run-search";
    settings: RunSettings;
    locations: Location[];
};

type SearchResponse =
    | { type: "search-result"; result: SearchResult }
    | { type: "search-error"; message: string };

self.addEventListener("message", async (event: MessageEvent<SearchRequest>) => {
    if (event.data.type !== "run-search") {
        return;
    }

    try {
        const settings = event.data.settings;
        await init();

        if (settings.mode === "parallel") {
            await init_parallel_runtime(settings.threads);
        }

        let parallelize = settings.mode === "parallel";
        const result = run_search(
            parallelize,
            settings.iterations,
            settings.seed,
            settings.threads,
            settings.chunkSize,
            event.data.locations,
        );

        const message: SearchResponse = { type: "search-result", result };
        self.postMessage(message);
    } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        const response: SearchResponse = { type: "search-error", message };
        self.postMessage(response);
    }
});