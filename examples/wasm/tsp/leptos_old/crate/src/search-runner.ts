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

export function runSearchOnce(settings: RunSettings): Promise<SearchResult> {
    return new Promise<SearchResult>((resolve, reject) => {
        const worker = new Worker(new URL("/src/search-worker.ts", import.meta.url), {
            type: "module"
        });

        const cleanup = () => {
            worker.terminate();
        };

        worker.addEventListener(
            "message",
            (event: MessageEvent) => {
                const data = event.data as
                    | { type: "search-result"; result: SearchResult }
                    | { type: "search-error"; message: string };

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

        worker.postMessage({ type: "run-search", settings });
    });
}