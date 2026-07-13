import initRayon, {
    init_parallel_runtime as initRayonRuntime,
    run_best_tour_par as runRayonPar,
} from "../pkg_rayon/orx_parallel_wasm_perf_rayon.js";
import initOrxRayon, {
    init_parallel_runtime as initOrxRayonRuntime,
    run_best_tour_par as runOrxRayonPar,
} from "../pkg_rayon_orx/orx_parallel_wasm_perf_rayon_orx.js";
import initOrx, {
    init_parallel_runtime as initOrxRuntime,
    run_best_tour_seq as runOrxSeq,
    run_best_tour_par as runOrxPar,
} from "../pkg_orx/orx_parallel_wasm_perf_orx.js";
import {
    type BenchmarkConfig,
    type BenchmarkReport,
    type VariantName,
    type VariantRunner,
    formatReportText,
    parseCsvInts,
    runSequentialMatrix,
    runVariantMatrix,
} from "./benchmark_core";

const DEFAULT_VARIANT: VariantName = "rayon";
const DEFAULT_NUM_THREADS = 4;
const VARIANT: VariantName = readVariantFromEnv(import.meta.env.PAR_POOL_VARIANT);
const NUM_THREADS = readNumThreadsFromEnv(import.meta.env.PAR_NUM_THREADS);

function readVariantFromEnv(raw: string | undefined): VariantName {
    if (!raw) {
        return DEFAULT_VARIANT;
    }

    const normalized = raw.trim().toLowerCase();
    if (normalized === "rayon" || normalized === "orx-rayon" || normalized === "orx") {
        return normalized;
    }

    console.warn(
        `unsupported PAR_POOL_VARIANT=${raw}; falling back to ${DEFAULT_VARIANT}. Supported values: rayon, orx-rayon, orx`,
    );
    return DEFAULT_VARIANT;
}

function readNumThreadsFromEnv(raw: string | undefined): number {
    if (!raw) {
        return DEFAULT_NUM_THREADS;
    }

    const parsed = Number.parseInt(raw.trim(), 10);
    if (!Number.isFinite(parsed) || parsed < 1 || parsed > 16) {
        console.warn(
            `unsupported PAR_NUM_THREADS=${raw}; falling back to ${DEFAULT_NUM_THREADS}. Supported range: 1..16`,
        );
        return DEFAULT_NUM_THREADS;
    }

    return parsed;
}

function mustElement<T extends HTMLElement>(id: string): T {
    const el = document.getElementById(id);
    if (!el) {
        throw new Error(`missing required element: #${id}`);
    }
    return el as T;
}

const ui = {
    status: mustElement<HTMLDivElement>("status"),
    chunkSize: mustElement<HTMLInputElement>("chunk-size"),
    cities: mustElement<HTMLInputElement>("cities"),
    iterations: mustElement<HTMLInputElement>("iterations"),
    warmups: mustElement<HTMLInputElement>("warmups"),
    runs: mustElement<HTMLInputElement>("runs"),
    seed: mustElement<HTMLInputElement>("seed"),
    run: mustElement<HTMLButtonElement>("run"),
    runSequential: mustElement<HTMLButtonElement>("run-sequential"),
    clear: mustElement<HTMLButtonElement>("clear"),
    output: mustElement<HTMLPreElement>("output"),
};

let busy = false;
let runtimePromise: Promise<[VariantName, VariantRunner]> | undefined;

function readyMessage(): string {
    return `Pool initialized: variant=${VARIANT}, threads=${NUM_THREADS}. Ready.`;
}

async function setup() {
    ui.run.addEventListener("click", () => {
        void runBenchmark();
    });

    ui.runSequential.addEventListener("click", () => {
        void runSequentialBenchmark();
    });

    ui.clear.addEventListener("click", () => {
        ui.output.textContent = "Output cleared.";
        ui.status.textContent = readyMessage();
    });

    ui.status.textContent = `Loading wasm module for ${VARIANT}...`;
    void preInitializeRuntime();
}

async function preInitializeRuntime() {
    try {
        await getRuntimeRunner();
        ui.status.textContent = readyMessage();
    } catch (err) {
        console.error("runtime initialization failed", err);
        ui.status.textContent = `Runtime initialization failed: ${String(err)}`;
    }
}

async function runBenchmark() {
    if (busy) {
        return;
    }

    busy = true;
    setControlsEnabled(false);

    try {
        const cfg = readConfig();
        const [variant, runner] = await getRuntimeRunner();

        const rows = await runVariantMatrix(variant, runner, cfg, (message) => {
            ui.status.textContent = message;
        });

        const report: BenchmarkReport = {
            config: {
                threads: cfg.threads,
                chunkSize: cfg.chunkSize,
                cityCounts: cfg.cityCounts,
                iterationCounts: cfg.iterationCounts,
                warmups: cfg.warmups,
                runs: cfg.runs,
                seed: cfg.seed.toString(),
            },
            rows,
        };

        ui.output.textContent = formatReportText(report);
        ui.status.textContent = "Benchmark completed.";
    } catch (err) {
        console.error("benchmark failed", err);
        ui.status.textContent = `Benchmark failed: ${String(err)}`;
    } finally {
        setControlsEnabled(true);
        busy = false;
    }
}

async function runSequentialBenchmark() {
    if (busy) {
        return;
    }

    busy = true;
    setControlsEnabled(false);

    try {
        const cfg = readConfig();
        await getRuntimeRunner();

        const rows = await runSequentialMatrix(runOrxSeq, cfg, (message) => {
            ui.status.textContent = message;
        });

        const report: BenchmarkReport = {
            config: {
                threads: cfg.threads,
                chunkSize: cfg.chunkSize,
                cityCounts: cfg.cityCounts,
                iterationCounts: cfg.iterationCounts,
                warmups: cfg.warmups,
                runs: cfg.runs,
                seed: cfg.seed.toString(),
            },
            rows,
        };

        ui.output.textContent = formatReportText(report);
        ui.status.textContent = "Sequential benchmark completed.";
    } catch (err) {
        console.error("sequential benchmark failed", err);
        ui.status.textContent = `Sequential benchmark failed: ${String(err)}`;
    } finally {
        setControlsEnabled(true);
        busy = false;
    }
}

async function createRunner(variant: VariantName): Promise<[VariantName, VariantRunner]> {
    switch (variant) {
        case "rayon": {
            await initRayon();
            return [
                variant,
                {
                    init_parallel_runtime: initRayonRuntime,
                    run_best_tour_par: runRayonPar,
                },
            ];
        }
        case "orx-rayon": {
            await initOrxRayon();
            return [
                variant,
                {
                    init_parallel_runtime: initOrxRayonRuntime,
                    run_best_tour_par: runOrxRayonPar,
                },
            ];
        }
        case "orx": {
            await initOrx();
            return [
                variant,
                {
                    init_parallel_runtime: initOrxRuntime,
                    run_best_tour_par: runOrxPar,
                },
            ];
        }
    }

    throw new Error(`unsupported variant: ${variant}`);
}

function readConfig(): BenchmarkConfig {
    const chunkSize = clampInt(ui.chunkSize.valueAsNumber, 1, 1_048_576, 1);
    const warmups = clampInt(ui.warmups.valueAsNumber, 0, 20, 2);
    const runs = clampInt(ui.runs.valueAsNumber, 1, 30, 5);
    const seedInput = clampInt(ui.seed.valueAsNumber, 1, 99_999_999, 42);

    const cityCounts = parseCsvInts(ui.cities.value, [50, 75]).map((x) => clampInt(x, 5, 200, 50));
    const iterationCounts = parseCsvInts(ui.iterations.value, [1000, 10000]).map((x) => clampInt(x, 1, 1_000_000, 1000));

    return {
        threads: NUM_THREADS,
        chunkSize,
        cityCounts,
        iterationCounts,
        warmups,
        runs,
        seed: BigInt(seedInput),
    };
}

function setControlsEnabled(enabled: boolean) {
    ui.chunkSize.disabled = !enabled;
    ui.cities.disabled = !enabled;
    ui.iterations.disabled = !enabled;
    ui.warmups.disabled = !enabled;
    ui.runs.disabled = !enabled;
    ui.seed.disabled = !enabled;
    ui.run.disabled = !enabled;
    ui.runSequential.disabled = !enabled;
    ui.clear.disabled = !enabled;
}

async function getRuntimeRunner(): Promise<[VariantName, VariantRunner]> {
    if (!runtimePromise) {
        runtimePromise = (async () => {
            const [variant, runner] = await createRunner(VARIANT);
            ui.status.textContent = `Initializing ${variant} runtime with ${NUM_THREADS} threads...`;
            await runner.init_parallel_runtime(NUM_THREADS);
            return [variant, runner];
        })();

        runtimePromise.catch(() => {
            runtimePromise = undefined;
        });
    }

    return runtimePromise;
}

function clampInt(value: number, min: number, max: number, fallback: number): number {
    if (!Number.isFinite(value)) {
        return fallback;
    }
    return Math.max(min, Math.min(max, Math.trunc(value)));
}

void setup();
