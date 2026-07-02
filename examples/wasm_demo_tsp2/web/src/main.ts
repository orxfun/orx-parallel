import init, {
    init_parallel_runtime,
    locations,
    parallel_runtime_info,
    run_best_tour_par,
    run_best_tour_seq
} from "../pkg/orx_parallel_wasm_demo_tsp2.js";

type RuntimeInfo = {
    configured_threads: number;
    spawned_workers: number;
};

type Location = { x: number; y: number };
type SearchChunkResult = {
    best_tour: number[];
    best_distance: number;
    iterations: number;
    elapsed_ms: number;
};
type SearchMode = "parallel" | "sequential";

type RunSettings = {
    mode: SearchMode;
    iterations: number;
    threads: number;
    seed: bigint;
    numCities: number;
};

type RunAggregate = {
    best_tour: number[];
    best_distance: number;
    iterations: number;
    elapsed_ms: number;
};

function mustElement<T extends HTMLElement>(id: string): T {
    const el = document.getElementById(id);
    if (!el) {
        throw new Error(`missing required element: #${id}`);
    }
    return el as T;
}

const ui = {
    status: mustElement<HTMLDivElement>("status"),
    iterations: mustElement<HTMLInputElement>("iterations"),
    threads: mustElement<HTMLInputElement>("threads"),
    seed: mustElement<HTMLInputElement>("seed"),
    numCities: mustElement<HTMLInputElement>("numCities"),
    runParallel: mustElement<HTMLButtonElement>("runParallel"),
    runSequential: mustElement<HTMLButtonElement>("runSequential"),
    reset: mustElement<HTMLButtonElement>("reset"),
    runOverlay: mustElement<HTMLDivElement>("runOverlay"),
    runTitle: mustElement<HTMLParagraphElement>("runTitle"),
    runSubtitle: mustElement<HTMLParagraphElement>("runSubtitle"),
    runElapsed: mustElement<HTMLParagraphElement>("runElapsed"),
    cancelRun: mustElement<HTMLButtonElement>("cancelRun"),
    bestDistance: mustElement<HTMLParagraphElement>("bestDistance"),
    elapsed: mustElement<HTMLParagraphElement>("elapsed"),
    ips: mustElement<HTMLParagraphElement>("ips"),
    canvas: mustElement<HTMLCanvasElement>("canvas")
};

const maybeCtx = ui.canvas.getContext("2d");

if (!maybeCtx) {
    throw new Error("failed to acquire canvas 2D context");
}

const ctx = maybeCtx;

const MIN_CITIES = 5;
const MAX_CITIES = 200;
const MIN_THREADS = 1;
const MAX_THREADS = 16;
const STARTUP_PARALLEL_RUNTIME_THREADS = 16;
const PARALLEL_RUNTIME_INIT_TIMEOUT_MS = 12_000;
const state = {
    points: [] as Location[],
    threadPoolReady: false,
    bestSoFar: null as RunAggregate | null,
    currentNumCities: MAX_CITIES,
    runTicker: undefined as number | undefined,
    runStartedAtMs: 0,
    cancelRequested: false
};

// Bootstraps wasm module, initial data and UI event wiring.
async function setupApp() {
    await init();

    state.currentNumCities = readNumCities();
    state.points = locations(state.currentNumCities) as Location[];
    drawPoints(state.points);

    wireUiHandlers();

    ui.status.textContent = "Initializing parallel runtime...";
    void initParallelRuntimeInBackground();
}

function wireUiHandlers() {

    ui.runParallel.addEventListener("click", async () => {
        await runSearch("parallel");
    });

    ui.runSequential.addEventListener("click", async () => {
        await runSearch("sequential");
    });

    ui.reset.addEventListener("click", () => {
        clearBest();
        drawPoints(state.points);
        ui.status.textContent = "Best tour reset. Ready for a fresh run.";
    });

    ui.numCities.addEventListener("change", () => {
        const numCities = readNumCities();
        state.points = locations(numCities) as Location[];
        clearBest();
        drawPoints(state.points);
        ui.status.textContent = `Updated problem size to ${numCities} cities.`;
    });

    ui.threads.addEventListener("change", () => {
        const threads = readThreads();
        ui.status.textContent = `Thread limit set to ${threads}.`;
    });

    ui.cancelRun.addEventListener("click", () => {
        state.cancelRequested = true;
        ui.cancelRun.disabled = true;
        ui.runSubtitle.textContent = "Cancelling... this takes effect after the current chunk finishes.";
    });
}

async function initParallelRuntimeInBackground() {
    try {
        await promiseWithTimeout(
            init_parallel_runtime(STARTUP_PARALLEL_RUNTIME_THREADS),
            PARALLEL_RUNTIME_INIT_TIMEOUT_MS,
            "parallel runtime init timed out"
        );
        state.threadPoolReady = true;

        try {
            const runtimeInfo = parallel_runtime_info() as RuntimeInfo;
            if (runtimeInfo.spawned_workers > 0) {
                ui.status.textContent = `Ready. Parallel runtime configured=${runtimeInfo.configured_threads}, spawned_workers=${runtimeInfo.spawned_workers}.`;
            } else {
                ui.status.textContent = `Ready with fallback: configured=${runtimeInfo.configured_threads}, spawned_workers=0. Parallel runs will execute inline on this platform.`;
            }
        } catch (diagErr) {
            console.warn("parallel_runtime_info failed", diagErr);
            ui.status.textContent = "Ready. Parallel runtime initialized, but worker diagnostics are unavailable.";
        }
    } catch (err) {
        state.threadPoolReady = false;
        ui.status.textContent = `Parallel runtime init failed: ${String(err)}. Sequential mode remains available.`;
    }
}

function promiseWithTimeout<T>(promise: Promise<T>, timeoutMs: number, message: string): Promise<T> {
    return Promise.race([
        promise,
        new Promise<T>((_, reject) => {
            window.setTimeout(() => reject(new Error(message)), timeoutMs);
        })
    ]);
}

function readRunSettings(mode: SearchMode): RunSettings {
    const iterations = Math.max(1, Number(ui.iterations.value) || 1);
    const threads = readThreads();
    const seedInput = Math.max(1, Number(ui.seed.value) || 1);
    return {
        mode,
        iterations,
        threads,
        seed: BigInt(Math.trunc(seedInput)),
        numCities: readNumCities()
    };
}

function setControlsDisabled(disabled: boolean) {
    ui.runParallel.disabled = disabled;
    ui.runSequential.disabled = disabled;
    ui.reset.disabled = disabled;
    ui.iterations.disabled = disabled;
    ui.threads.disabled = disabled;
    ui.seed.disabled = disabled;
    ui.numCities.disabled = disabled;
}

function ensurePointsForCities(numCities: number) {
    if (state.points.length === numCities) {
        return;
    }

    state.points = locations(numCities) as Location[];
    clearBest();
    drawPoints(state.points);
}

async function runSearch(mode: SearchMode) {
    const settings = readRunSettings(mode);
    ensurePointsForCities(settings.numCities);

    setControlsDisabled(true);
    setRunningView(settings.mode, true);
    await allowRunningOverlayToRender();

    ui.status.textContent = settings.mode === "parallel" ? "Running parallel search..." : "Running sequential search...";

    try {
        if (settings.mode === "parallel" && !state.threadPoolReady) {
            throw new Error("parallel runtime is not initialized");
        }

        let remaining = settings.iterations;
        let startIndex = 0;
        let runElapsedMs = 0;
        let runBest: SearchChunkResult | null = null;
        const chunkSize = chooseChunkSize(settings.mode, settings.numCities, settings.iterations);

        while (remaining > 0) {
            if (state.cancelRequested) {
                break;
            }

            const thisChunk = Math.min(remaining, chunkSize);
            const chunkResult = runSearchChunk(settings, thisChunk, startIndex);

            runElapsedMs += chunkResult.elapsed_ms;
            startIndex += thisChunk;
            remaining -= thisChunk;

            const isRunBest = !runBest || chunkResult.best_distance < runBest.best_distance;
            if (isRunBest) {
                runBest = chunkResult;
            }

            const currentRunBest = runBest ?? chunkResult;

            if (!state.bestSoFar || currentRunBest.best_distance < state.bestSoFar.best_distance) {
                state.bestSoFar = toAggregate(currentRunBest, startIndex, runElapsedMs);
                drawTour(state.points, state.bestSoFar.best_tour);
            }

            ui.runSubtitle.textContent = `Processed ${startIndex.toLocaleString()} / ${settings.iterations.toLocaleString()} iterations...`;
            await nextPaint();
        }

        if (startIndex === 0) {
            throw new Error(`${settings.mode} run produced no work`);
        }

        if (!runBest) {
            throw new Error(`${settings.mode} run produced no result`);
        }

        const summary = toAggregate(runBest, startIndex, runElapsedMs);
        updateStats(summary);

        if (state.cancelRequested) {
            ui.status.textContent = `${settings.mode === "parallel" ? "Parallel" : "Sequential"} run cancelled after ${startIndex.toLocaleString()} iterations.`;
        } else if (state.bestSoFar && runBest.best_distance <= state.bestSoFar.best_distance) {
            ui.status.textContent = `${settings.mode === "parallel" ? "Parallel" : "Sequential"} run completed (${startIndex.toLocaleString()} iterations, ${runElapsedMs.toFixed(1)} ms).`;
        } else {
            ui.status.textContent = `${settings.mode === "parallel" ? "Parallel" : "Sequential"} run completed (${startIndex.toLocaleString()} iterations, ${runElapsedMs.toFixed(1)} ms); best tour unchanged.`;
        }
    } catch (err) {
        console.error("runSearch failed", err);
        ui.status.textContent = `Error in ${settings.mode} run: ${String(err)}`;
    } finally {
        setRunningView(settings.mode, false);
        setControlsDisabled(false);
    }
}

function runSearchChunk(settings: RunSettings, chunkIterations: number, startIndex: number): SearchChunkResult {
    if (settings.mode === "parallel") {
        return run_best_tour_par(
            chunkIterations,
            settings.seed,
            settings.threads,
            settings.numCities,
            BigInt(startIndex)
        ) as SearchChunkResult;
    }

    return run_best_tour_seq(
        chunkIterations,
        settings.seed,
        settings.numCities,
        BigInt(startIndex)
    ) as SearchChunkResult;
}

function toAggregate(best: SearchChunkResult, iterations: number, elapsedMs: number): RunAggregate {
    return {
        best_tour: best.best_tour,
        best_distance: best.best_distance,
        iterations,
        elapsed_ms: elapsedMs
    };
}

function chooseChunkSize(mode: SearchMode, numCities: number, iterations: number) {
    const base = mode === "parallel" ? 400 : 240;
    const scale = Math.max(1, Math.floor(220 / Math.max(numCities, 5)));
    return Math.max(8, Math.min(iterations, base * scale));
}

function setRunningView(mode: SearchMode, running: boolean) {
    if (running) {
        state.cancelRequested = false;
        state.runStartedAtMs = performance.now();
        ui.runTitle.textContent = mode === "parallel" ? "Running parallel search..." : "Running sequential search...";
        ui.runSubtitle.textContent = "Evaluating tours with 2-opt local search. Larger instances can take several minutes.";
        ui.runElapsed.textContent = "Elapsed: 0.0s";
        ui.cancelRun.disabled = false;
        ui.runOverlay.classList.add("active");
        ui.runOverlay.setAttribute("aria-hidden", "false");

        if (state.runTicker !== undefined) {
            window.clearInterval(state.runTicker);
        }

        state.runTicker = window.setInterval(() => {
            const secs = (performance.now() - state.runStartedAtMs) / 1000;
            ui.runElapsed.textContent = `Elapsed: ${secs.toFixed(1)}s`;
        }, 200);
        return;
    }

    if (state.runTicker !== undefined) {
        window.clearInterval(state.runTicker);
        state.runTicker = undefined;
    }

    ui.cancelRun.disabled = true;
    ui.runOverlay.classList.remove("active");
    ui.runOverlay.setAttribute("aria-hidden", "true");
}

function nextPaint() {
    return new Promise<void>((resolve) => {
        requestAnimationFrame(() => resolve());
    });
}

async function allowRunningOverlayToRender() {
    // Two frames + a tiny timeout makes spinner start reliably before sync wasm work blocks the UI thread.
    await nextPaint();
    await nextPaint();
    await new Promise<void>((resolve) => window.setTimeout(resolve, 24));
}

function readNumCities() {
    const parsed = ui.numCities.valueAsNumber;

    if (!Number.isFinite(parsed)) {
        ui.numCities.value = String(state.currentNumCities);
        return state.currentNumCities;
    }

    const numCities = Math.max(MIN_CITIES, Math.min(MAX_CITIES, Math.trunc(parsed)));
    state.currentNumCities = numCities;
    ui.numCities.value = String(numCities);
    return numCities;
}

function readThreads() {
    const parsed = ui.threads.valueAsNumber;

    if (!Number.isFinite(parsed)) {
        ui.threads.value = String(MIN_THREADS);
        return MIN_THREADS;
    }

    const threads = Math.max(MIN_THREADS, Math.min(MAX_THREADS, Math.trunc(parsed)));
    ui.threads.value = String(threads);
    return threads;
}

function clearBest() {
    state.bestSoFar = null;
    ui.bestDistance.textContent = "-";
    ui.elapsed.textContent = "-";
    ui.ips.textContent = "-";
}

function updateStats(result: RunAggregate) {
    ui.bestDistance.textContent = result.best_distance.toFixed(3);
    ui.elapsed.textContent = `${result.elapsed_ms.toFixed(1)} ms`;
    const ips = result.iterations / Math.max(result.elapsed_ms / 1000, 1e-9);
    ui.ips.textContent = ips.toFixed(0);
}

function drawPoints(locations: Location[]) {
    ctx.clearRect(0, 0, ui.canvas.width, ui.canvas.height);
    const mapped = mapPoints(locations);

    for (const p of mapped) {
        ctx.beginPath();
        ctx.fillStyle = "#0f766e";
        ctx.arc(p.x, p.y, 5, 0, Math.PI * 2);
        ctx.fill();
    }
}

function drawTour(locations: Location[], tour: number[]) {
    drawPoints(locations);
    if (tour.length === 0) {
        return;
    }

    const mapped = mapPoints(locations);
    ctx.beginPath();
    ctx.strokeStyle = "#f59e0b";
    ctx.lineWidth = 2;

    const first = mapped[tour[0]];
    ctx.moveTo(first.x, first.y);

    for (const idx of tour.slice(1)) {
        const p = mapped[idx];
        ctx.lineTo(p.x, p.y);
    }

    ctx.lineTo(first.x, first.y);
    ctx.stroke();
}

function mapPoints(locations: Location[]) {
    const pad = 28;
    const xs = locations.map((p) => p.x);
    const ys = locations.map((p) => p.y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    const spanX = Math.max(maxX - minX, 1);
    const spanY = Math.max(maxY - minY, 1);

    return locations.map((p) => ({
        x: pad + ((p.x - minX) / spanX) * (ui.canvas.width - pad * 2),
        y: ui.canvas.height - (pad + ((p.y - minY) / spanY) * (ui.canvas.height - pad * 2))
    }));
}

void setupApp();
