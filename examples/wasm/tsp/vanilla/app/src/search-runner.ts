import { ParallelWorker } from "orx-parallel-web";
import bindingsUrl from "../pkg/wasm_bindings.js?url";
import type { Location, RunSettings, SearchResult } from "./shared-types";

type TspComputations = {
    run_search: (
        parallelize: boolean,
        iterations: number,
        seed: bigint,
        threads: number,
        chunkSize: number,
        locations: Location[]
    ) => SearchResult;
};

const searchWorker = new ParallelWorker<TspComputations>({
    bindingsUrl,
    methods: ["run_search"],
    threads: 0
});

export function initializeSearchWorker(): Promise<void> {
    return searchWorker.ready();
}

export function terminateSearchWorker(): void {
    searchWorker.terminate();
}

export function runSearchAlgorithm(settings: RunSettings, locations: Location[]): Promise<SearchResult> {
    return searchWorker.call("run_search", [
        true,
        settings.iterations,
        settings.seed,
        settings.threads,
        settings.chunkSize,
        locations
    ]);
}
