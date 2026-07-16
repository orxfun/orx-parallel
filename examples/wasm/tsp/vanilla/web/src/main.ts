import init, {
    init_parallel_runtime,
    locations,
    run_best_tour_par,
    run_best_tour_seq
} from "../pkg/orx_parallel_wasm_tsp_vanilla.js";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import "highlight.js/styles/github-dark.css";

hljs.registerLanguage("rust", rust);

type Location = { x: number; y: number };
type SearchResult = {
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
    chunkSize: number;
    seed: bigint;
    numCities: number;
};

type RunAggregate = {
    best_tour: number[];
    best_distance: number;
    iterations: number;
    elapsed_ms: number;
};

const runBestTourPar = run_best_tour_par as unknown as (
    iterations: number,
    seed: bigint,
    threads: number,
    chunkSize: number,
    numCities: number
) => SearchResult;

const MIN_CITIES = 5;
const MAX_CITIES = 200;
const MIN_THREADS = 1;
const MAX_THREADS = 16;
const DEFAULT_STARTUP_THREADS = 16;
const CITY_NODE_COLOR = readCssColor("--city-node", "#f59e0b");
const TOUR_LINE_COLOR = readCssColor("--tour-line", "#1d4ed8");
const CANVAS_BACKGROUND_COLOR = readCssColor("--code-block-bg", "#0f172a");

function mustElement<T extends HTMLElement>(id: string): T {
    const el = document.getElementById(id);
    if (!el) {
        throw new Error(`missing required element: #${id}`);
    }
    return el as T;
}

function readCssColor(variableName: string, fallback: string) {
    const value = getComputedStyle(document.documentElement).getPropertyValue(variableName).trim();
    return value || fallback;
}

const ui = {
    status: mustElement<HTMLDivElement>("status"),
    iterations: mustElement<HTMLInputElement>("iterations"),
    threads: mustElement<HTMLInputElement>("threads"),
    chunkSize: mustElement<HTMLInputElement>("chunkSize"),
    seed: mustElement<HTMLInputElement>("seed"),
    numCities: mustElement<HTMLInputElement>("numCities"),
    runParallel: mustElement<HTMLButtonElement>("runParallel"),
    runSequential: mustElement<HTMLButtonElement>("runSequential"),
    reset: mustElement<HTMLButtonElement>("reset"),
    runOverlay: mustElement<HTMLDivElement>("runOverlay"),
    runTitle: mustElement<HTMLParagraphElement>("runTitle"),
    runSubtitle: mustElement<HTMLParagraphElement>("runSubtitle"),
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

const state = {
    points: [] as Location[],
    threadPoolReady: false,
    bestSoFar: null as RunAggregate | null,
    currentNumCities: 50,
    runStartedAtMs: 0
};

async function setupApp() {
    await init();

    const startupThreads = readStartupThreadsFromEnv();

    try {
        await init_parallel_runtime(startupThreads);
        state.threadPoolReady = true;
        ui.status.textContent = `Ready. Parallel runtime initialized with ${startupThreads} threads.`;
    } catch (err) {
        state.threadPoolReady = false;
        ui.status.textContent = `Parallel runtime init failed: ${String(err)}. Sequential mode remains available.`;
    }

    state.currentNumCities = readNumCities();
    state.points = generatePoints(readSeed(), state.currentNumCities);
    drawPoints(state.points);
    ui.chunkSize.value = String(readChunkSize());
    highlightCodeBlocks();

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
        state.points = generatePoints(readSeed(), numCities);
        clearBest();
        drawPoints(state.points);
        ui.status.textContent = `Updated problem size to ${numCities} cities.`;
    });

    ui.seed.addEventListener("change", () => {
        const seed = readSeed();
        const numCities = readNumCities();
        state.points = generatePoints(seed, numCities);
        clearBest();
        drawPoints(state.points);
        ui.status.textContent = `Updated city seed to ${seed.toString()}.`;
    });

    ui.threads.addEventListener("change", () => {
        const threads = readThreads();
        ui.status.textContent = `Thread limit set to ${threads}.`;
    });

    ui.chunkSize.addEventListener("change", () => {
        const chunkSize = readChunkSize();
        ui.status.textContent = `Chunk size set to ${chunkSize}.`;
    });
}

function readStartupThreadsFromEnv() {
    const raw = readEnvValue("ORX_PARALLEL_MAX_NUM_THREADS");
    if (!raw) {
        return DEFAULT_STARTUP_THREADS;
    }

    const parsed = Number.parseInt(raw.trim(), 10);
    if (!Number.isFinite(parsed)) {
        return DEFAULT_STARTUP_THREADS;
    }

    return Math.max(MIN_THREADS, Math.min(MAX_THREADS, Math.trunc(parsed)));
}

function readEnvValue(key: string) {
    return (import.meta.env as Record<string, string | undefined>)[key];
}

function highlightCodeBlocks() {
    document.querySelectorAll<HTMLPreElement>(".code-block code").forEach((block) => {
        block.classList.add("language-rust");
        hljs.highlightElement(block);
    });
}

function readSeed() {
    const seedInput = Math.max(1, Number(ui.seed.value) || 1);
    return BigInt(Math.trunc(seedInput));
}

function generatePoints(seed: bigint, numCities: number) {
    return locations(seed, numCities) as Location[];
}

function readRunSettings(mode: SearchMode): RunSettings {
    const iterations = Math.max(1, Number(ui.iterations.value) || 1);
    const threads = readThreads();
    const chunkSize = readChunkSize();
    return {
        mode,
        iterations,
        threads,
        chunkSize,
        seed: readSeed(),
        numCities: readNumCities()
    };
}

function setControlsDisabled(disabled: boolean) {
    ui.runParallel.disabled = disabled;
    ui.runSequential.disabled = disabled;
    ui.reset.disabled = disabled;
    ui.iterations.disabled = disabled;
    ui.threads.disabled = disabled;
    ui.chunkSize.disabled = disabled;
    ui.seed.disabled = disabled;
    ui.numCities.disabled = disabled;
}

function ensurePointsForCities(numCities: number) {
    if (state.points.length === numCities) {
        return;
    }

    state.points = generatePoints(readSeed(), numCities);
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

        const result = runSearchOnce(settings);

        if (!state.bestSoFar || result.best_distance < state.bestSoFar.best_distance) {
            state.bestSoFar = toAggregate(result);
            drawTour(state.points, state.bestSoFar.best_tour);
        }

        updateStats(toAggregate(result));
        ui.runSubtitle.textContent = `Processed ${settings.iterations.toLocaleString()} iterations in one call.`;
        ui.status.textContent = `${settings.mode === "parallel" ? "Parallel" : "Sequential"} run completed.`;
    } catch (err) {
        ui.status.textContent = `Error: ${String(err)}`;
    } finally {
        setRunningView(settings.mode, false);
        setControlsDisabled(false);
    }
}

function runSearchOnce(settings: RunSettings): SearchResult {
    if (settings.mode === "parallel") {
        return runBestTourPar(
            settings.iterations,
            settings.seed,
            settings.threads,
            settings.chunkSize,
            settings.numCities
        );
    } else {
        return run_best_tour_seq(settings.iterations, settings.seed, settings.numCities) as SearchResult;
    }
}

function toAggregate(best: SearchResult): RunAggregate {
    return {
        best_tour: best.best_tour,
        best_distance: best.best_distance,
        iterations: best.iterations,
        elapsed_ms: best.elapsed_ms
    };
}

function setRunningView(mode: SearchMode, running: boolean) {
    if (running) {
        state.runStartedAtMs = performance.now();
        ui.runTitle.textContent = mode === "parallel" ? "Running parallel search..." : "Running sequential search...";
        ui.runSubtitle.textContent = "Evaluating tours with 2-opt local search. Larger instances can take longer.";
        ui.runOverlay.classList.add("active");
        ui.runOverlay.setAttribute("aria-hidden", "false");

        return;
    }

    ui.runOverlay.classList.remove("active");
    ui.runOverlay.setAttribute("aria-hidden", "true");
}

function nextPaint() {
    return new Promise<void>((resolve) => {
        requestAnimationFrame(() => resolve());
    });
}

async function allowRunningOverlayToRender() {
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

function readChunkSize() {
    const parsed = ui.chunkSize.valueAsNumber;

    if (!Number.isFinite(parsed)) {
        ui.chunkSize.value = "0";
        return 0;
    }

    const chunkSize = Math.max(0, Math.trunc(parsed));
    ui.chunkSize.value = String(chunkSize);
    return chunkSize;
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
    ctx.fillStyle = CANVAS_BACKGROUND_COLOR;
    ctx.fillRect(0, 0, ui.canvas.width, ui.canvas.height);
    const mapped = mapPoints(locations);

    for (const p of mapped) {
        ctx.beginPath();
        ctx.fillStyle = CITY_NODE_COLOR;
        ctx.arc(p.x, p.y, 6, 0, Math.PI * 2);
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
    ctx.strokeStyle = TOUR_LINE_COLOR;
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
