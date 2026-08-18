export type Location = { x: number; y: number };

export type SearchResult = {
    best_tour: number[];
    best_distance: number;
    iterations: number;
    elapsed_ms: number;
};

export type RunSettings = {
    iterations: number;
    threads: number;
    chunkSize: number;
    seed: bigint;
    numCities: number;
};
