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
    run_best_tour_par as runOrxPar,
} from "../pkg_orx/orx_parallel_wasm_perf_orx.js";
import {
    type BenchmarkConfig,
    type BenchmarkReport,
    type VariantName,
    type VariantRunner,
    formatReportText,
    parseCsvInts,
    runVariantMatrix,
} from "./benchmark_core";

const VARIANT: VariantName = "rayon";

function mustElement<T extends HTMLElement>(id: string): T {
    const el = document.getElementById(id);
    if (!el) {
        throw new Error(`missing required element: #${id}`);
    }
    return el as T;
}

const ui = {
    status: mustElement<HTMLDivElement>("status"),
    threads: mustElement<HTMLInputElement>("threads"),
    cities: mustElement<HTMLInputElement>("cities"),
    iterations: mustElement<HTMLInputElement>("iterations"),
    warmups: mustElement<HTMLInputElement>("warmups"),
    runs: mustElement<HTMLInputElement>("runs"),
    seed: mustElement<HTMLInputElement>("seed"),
    run: mustElement<HTMLButtonElement>("run"),
    clear: mustElement<HTMLButtonElement>("clear"),
    output: mustElement<HTMLPreElement>("output"),
};

let busy = false;

async function setup() {
    ui.run.addEventListener("click", () => {
        void runBenchmark();
    });

    ui.clear.addEventListener("click", () => {
        ui.output.textContent = "Output cleared.";
        ui.status.textContent = "Ready.";
    });
}

async function runBenchmark() {
    if (busy) {
        return;
    }

    busy = true;
    setControlsEnabled(false);

    try {
        const cfg = readConfig();
        ui.status.textContent = `Loading wasm module for ${VARIANT}...`;

        const [variant, runner] = await createRunner(VARIANT);

        ui.status.textContent = `Initializing ${variant} runtime...`;
        await runner.init_parallel_runtime(cfg.threads);
        const rows = await runVariantMatrix(variant, runner, cfg, (message) => {
            ui.status.textContent = message;
        });

        const report: BenchmarkReport = {
            config: {
                threads: cfg.threads,
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
    const threads = clampInt(ui.threads.valueAsNumber, 1, 16, 4);
    const warmups = clampInt(ui.warmups.valueAsNumber, 0, 20, 2);
    const runs = clampInt(ui.runs.valueAsNumber, 1, 30, 5);
    const seedInput = clampInt(ui.seed.valueAsNumber, 1, 99_999_999, 42);

    const cityCounts = parseCsvInts(ui.cities.value, [50, 75]).map((x) => clampInt(x, 5, 200, 50));
    const iterationCounts = parseCsvInts(ui.iterations.value, [1000, 10000]).map((x) => clampInt(x, 1, 1_000_000, 1000));

    return {
        threads,
        cityCounts,
        iterationCounts,
        warmups,
        runs,
        seed: BigInt(seedInput),
    };
}

function setControlsEnabled(enabled: boolean) {
    ui.threads.disabled = !enabled;
    ui.cities.disabled = !enabled;
    ui.iterations.disabled = !enabled;
    ui.warmups.disabled = !enabled;
    ui.runs.disabled = !enabled;
    ui.seed.disabled = !enabled;
    ui.run.disabled = !enabled;
    ui.clear.disabled = !enabled;
}

function clampInt(value: number, min: number, max: number, fallback: number): number {
    if (!Number.isFinite(value)) {
        return fallback;
    }
    return Math.max(min, Math.min(max, Math.trunc(value)));
}

void setup();
