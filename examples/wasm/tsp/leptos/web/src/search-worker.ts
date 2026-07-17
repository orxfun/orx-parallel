import init, {
    init_parallel_runtime,
    run_best_tour_par,
    run_best_tour_seq
} from "../pkg/orx_parallel_wasm_tsp_leptos.js";

type SearchMode = "parallel" | "sequential";

type RunSettings = {
    mode: SearchMode;
    iterations: number;
    threads: number;
    chunk_size: number;
    seed: number;
    num_cities: number;
};

type SearchResult = {
    best_tour: number[];
    best_distance: number;
    iterations: number;
    elapsed_ms: number;
};

let wasmInitPromise: Promise<void> | null = null;

self.addEventListener("message", async (event: MessageEvent) => {
    const payload = event.data as { type: "run-search"; settings: RunSettings };

    try {
        await ensureWasmInitialized();
        const result = await runSearch(payload.settings);
        self.postMessage({ type: "search-result", result });
    } catch (err) {
        self.postMessage({
            type: "search-error",
            message: err instanceof Error ? err.message : String(err)
        });
    }
});

async function ensureWasmInitialized() {
    if (!wasmInitPromise) {
        wasmInitPromise = init().then(() => undefined);
    }

    await wasmInitPromise;
}

async function runSearch(settings: RunSettings): Promise<SearchResult> {
    if (settings.mode === "parallel") {
        await init_parallel_runtime(settings.threads);
        return run_best_tour_par(
            settings.iterations,
            settings.seed,
            settings.threads,
            settings.chunk_size,
            settings.num_cities
        ) as SearchResult;
    }

    return run_best_tour_seq(settings.iterations, settings.seed, settings.num_cities) as SearchResult;
}