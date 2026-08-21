import { ParallelWorker } from "orx-parallel-wasm";
import bindingsUrl from "../pkg/components.js?url";
import { normalizeSeed, type Location, type RunSettings, type SearchResult } from "./shared-types.js";

type TspComputations = {
    run_search: (
        iterations: number,
        seed: bigint,
        threads: number,
        chunkSize: number,
        locations: Location[]
    ) => SearchResult;
};

let searchWorker: ParallelWorker<TspComputations> | undefined;

export async function createSearchWorker(threads: number): Promise<void> {
    searchWorker = new ParallelWorker<TspComputations>({
        bindingsUrl,
        methods: ["run_search"],
        threads,
    });
    await searchWorker.ready();
}

export function terminateSearchWorker(): void {
    searchWorker?.terminate();
    searchWorker = undefined;
}

export function runSearchAlgorithm(settings: RunSettings, locations: Location[]): Promise<SearchResult> {
    if (!searchWorker) {
        return Promise.reject(new Error("search worker is not initialized"));
    }

    return searchWorker.call("run_search", [
        settings.iterations,
        normalizeSeed(settings.seed),
        settings.threads,
        settings.chunkSize,
        locations
    ]);
}

(globalThis as typeof globalThis & { runSearchAlgorithm: typeof runSearchAlgorithm }).runSearchAlgorithm = runSearchAlgorithm;
