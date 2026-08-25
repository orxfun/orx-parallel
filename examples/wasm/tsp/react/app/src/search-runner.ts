import { ParallelWorker } from "orx-parallel-wasm";
import bindingsUrl from "../pkg/wasm_bindings.js?url";
import type { Location, RunSettings, SearchResult } from "./shared-types";

type TspComputations = {
    run_search: (
        iterations: number,
        seed: bigint,
        threads: number,
        chunkSize: number,
        locations: Location[]
    ) => SearchResult;
};

export class SearchWorker {
    constructor(private readonly worker: ParallelWorker<TspComputations>) {
    }

    runSearchAlgorithm(settings: RunSettings, locations: Location[]): Promise<SearchResult> {
        return this.worker.call("run_search", [
            settings.iterations,
            settings.seed,
            settings.threads,
            settings.chunkSize,
            locations
        ]);
    }

    terminate(): void {
        this.worker.terminate();
    }
}

export async function createSearchWorker(threads: number): Promise<SearchWorker> {
    const searchWorker = new ParallelWorker<TspComputations>({
        bindingsUrl,
        methods: ["run_search"],
        threads,
    });
    await searchWorker.ready();
    return new SearchWorker(searchWorker);
}
