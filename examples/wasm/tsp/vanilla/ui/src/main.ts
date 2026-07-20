import init, { locations } from "../pkg/wasm_bindings.js";
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

const MAX_CITIES = 200;
const MAX_THREADS = 16;
const CITY_NODE_COLOR = readCssColor("--city-node");
const TOUR_LINE_COLOR = readCssColor("--tour-line");
const CANVAS_BACKGROUND_COLOR = readCssColor("--code-block-bg");

function readCssColor(variableName: string) {
    getComputedStyle(document.documentElement).getPropertyValue(variableName).trim()
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
    runElapsed: mustElement<HTMLParagraphElement>("runElapsed"),
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

// input readers

function readNumCities(): number {
    const parsed = ui.numCities.valueAsNumber;

    if (!Number.isFinite(parsed)) {
        ui.numCities.value = String(state.currentNumCities);
        return state.currentNumCities;
    }

    const numCities = Math.max(3, Math.min(MAX_CITIES, Math.trunc(parsed)));
    state.currentNumCities = numCities;
    ui.numCities.value = String(numCities);
    return numCities;
}

function readThreads() {
    const parsed = ui.threads.valueAsNumber;

    if (!Number.isFinite(parsed)) {
        ui.threads.value = String(0);
        return 0;
    }

    const threads = Math.max(0, Math.min(MAX_THREADS, Math.trunc(parsed)));
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

// state

const state = {
    points: [] as Location[],
    bestSoFar: null as RunAggregate | null,
    currentNumCities: 50,
    runStartedAtMs: 0,
    runTicker: undefined as number | undefined
};

// set up

async function setupApp() {
    await init();

    state.currentNumCities = readNumCities();
    state.points = generatePoints(readSeed(), state.currentNumCities);
    drawPoints(state.points);
    ui.chunkSize.value = String(readChunkSize());
    highlightCodeBlocks();
    ui.status.textContent = "Ready";
}

setupApp();
