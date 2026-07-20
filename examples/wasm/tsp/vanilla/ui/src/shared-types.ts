export type Location = { x: number; y: number };

export type SearchResult = {
    best_tour: number[];
    best_distance: number;
    iterations: number;
    elapsed_ms: number;
};

export type SearchMode = "parallel" | "sequential";

export type RunSettings = {
    mode: SearchMode;
    iterations: number;
    threads: number;
    chunkSize: number;
    seed: bigint;
    numCities: number;
};

export type SearchRequest = {
    type: "run-search";
    settings: RunSettings;
    locations: Location[];
};

export type SearchResponse =
    | { type: "search-result"; result: SearchResult }
    | { type: "search-error"; message: string };