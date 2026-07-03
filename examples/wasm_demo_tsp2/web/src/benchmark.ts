import init, {
    init_parallel_runtime,
    parallel_runtime_info,
    run_best_tour_seq,
    run_parallel_benchmark_report
} from "../pkg/orx_parallel_wasm_demo_tsp2.js";

type RuntimeInfo = {
    configured_threads: number;
    spawned_workers: number;
};

type SequentialChunkResult = {
    best_tour: number[];
    best_distance: number;
    iterations: number;
    elapsed_ms: number;
};

type ParallelBenchmarkReport = {
    trials: number;
    iterations_per_trial: number;
    threads: number;
    chunk_size: number;
    num_cities: number;
    median_ms: number;
    p95_ms: number;
    mean_ms: number;
    min_ms: number;
    max_ms: number;
    perf: {
        tasks_enqueued: number;
        tasks_run_by_workers: number;
        tasks_run_by_main: number;
        notify_count: number;
        queue_depth_high_water: number;
    };
    perf_extended: {
        queue_pop_count: number;
        queue_empty_poll_count: number;
        main_assist_time_ns: number;
        state_try_lock_fail_count: number;
        state_try_lock_spin_iters: number;
        completion_notify_count: number;
        main_assist_attempt_count: number;
        main_assist_success_count: number;
        worker_runs_by_id: number[];
    };
    trial_samples_ms: number[];
};

type Summary = {
    median_ms: number;
    p95_ms: number;
    mean_ms: number;
    min_ms: number;
    max_ms: number;
};

const CONFIG = {
    trials: 20,
    iterations: 10_000,
    threads: 16,
    chunkSizes: [0, 1, 2, 4, 8, 16, 32, 64, 128, 256],
    numCities: 50,
    seed: 42n,
};

function mustElement<T extends HTMLElement>(id: string): T {
    const el = document.getElementById(id);
    if (!el) {
        throw new Error(`missing #${id} element`);
    }
    return el as T;
}

const outputEl = mustElement<HTMLPreElement>("output");

function percentile(sortedSamples: number[], p: number): number {
    if (sortedSamples.length === 0) {
        return 0;
    }
    const idx = Math.floor((sortedSamples.length - 1) * p / 100);
    return sortedSamples[idx];
}

function summarize(samples: number[]): Summary {
    const sorted = [...samples].sort((a, b) => a - b);
    const sum = samples.reduce((acc, x) => acc + x, 0);
    return {
        median_ms: percentile(sorted, 50),
        p95_ms: percentile(sorted, 95),
        mean_ms: samples.length ? sum / samples.length : 0,
        min_ms: sorted[0] ?? 0,
        max_ms: sorted[sorted.length - 1] ?? 0,
    };
}

async function runSequentialTrials(): Promise<{ wall: Summary; wasm: Summary }> {
    const wallSamples: number[] = [];
    const wasmSamples: number[] = [];

    let startIndex = 0n;
    for (let i = 0; i < CONFIG.trials; i++) {
        const startedAt = performance.now();
        const result = run_best_tour_seq(
            CONFIG.iterations,
            CONFIG.seed,
            CONFIG.numCities,
            startIndex,
        ) as SequentialChunkResult;
        const wallMs = performance.now() - startedAt;

        wallSamples.push(wallMs);
        wasmSamples.push(result.elapsed_ms);
        startIndex += BigInt(CONFIG.iterations);
    }

    return {
        wall: summarize(wallSamples),
        wasm: summarize(wasmSamples),
    };
}

async function main() {
    outputEl.textContent = "Loading wasm module...";
    await init();

    outputEl.textContent = "Initializing parallel runtime...";
    await init_parallel_runtime(CONFIG.threads);

    const runtimeInfo = parallel_runtime_info() as RuntimeInfo;

    outputEl.textContent = "Running benchmark trials...";

    const sequential = await runSequentialTrials();
    const parallel: ParallelBenchmarkReport[] = [];
    for (const chunkSize of CONFIG.chunkSizes) {
        const report = run_parallel_benchmark_report(
            CONFIG.trials,
            CONFIG.iterations,
            CONFIG.seed,
            CONFIG.threads,
            chunkSize,
            CONFIG.numCities,
        ) as ParallelBenchmarkReport;
        parallel.push(report);
    }

    const report = {
        config: {
            ...CONFIG,
            seed: CONFIG.seed.toString(),
        },
        runtime: runtimeInfo,
        sequential,
        parallel,
    };

    console.log("wasm_demo_tsp2 benchmark report", report);
    outputEl.textContent = JSON.stringify(report, null, 2);
}

void main().catch((err) => {
    console.error("benchmark failed", err);
    outputEl.textContent = `benchmark failed: ${String(err)}`;
});
