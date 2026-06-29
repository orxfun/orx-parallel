import init, {
    init_thread_pool,
    locations,
    run_best_tour,
    run_best_tour_seq
} from "../pkg/orx_parallel_wasm_tour_demo.js";

type Location = { x: number; y: number };
type Result = {
    best_tour: number[];
    best_distance: number;
    iterations: number;
    elapsed_ms: number;
};

const statusEl = document.getElementById("status") as HTMLDivElement;
const iterationsEl = document.getElementById("iterations") as HTMLInputElement;
const threadsEl = document.getElementById("threads") as HTMLInputElement;
const seedEl = document.getElementById("seed") as HTMLInputElement;
const numCitiesEl = document.getElementById("numCities") as HTMLInputElement;
const runParallelEl = document.getElementById("runParallel") as HTMLButtonElement;
const runSequentialEl = document.getElementById("runSequential") as HTMLButtonElement;
const resetEl = document.getElementById("reset") as HTMLButtonElement;
const bestDistanceEl = document.getElementById("bestDistance") as HTMLParagraphElement;
const elapsedEl = document.getElementById("elapsed") as HTMLParagraphElement;
const ipsEl = document.getElementById("ips") as HTMLParagraphElement;
const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const maybeCtx = canvas.getContext("2d");

if (!maybeCtx) {
    throw new Error("failed to acquire canvas 2D context");
}

const ctx = maybeCtx;

let points: Location[] = [];
let initialized = false;
let bestSoFar: Result | null = null;

const MIN_CITIES = 5;
const MAX_CITIES = 200;
let currentNumCities = MAX_CITIES;

async function setup() {
    await init();
    currentNumCities = readNumCities();
    points = locations(currentNumCities) as Location[];
    drawPoints(points);
    statusEl.textContent = "Ready. Run parallel or sequential search.";
}

runParallelEl.addEventListener("click", async () => {
    await runSearch("parallel");
});

runSequentialEl.addEventListener("click", async () => {
    await runSearch("sequential");
});

resetEl.addEventListener("click", () => {
    clearBest();
    drawPoints(points);
    statusEl.textContent = "Best tour reset. Ready for a fresh run.";
});

numCitiesEl.addEventListener("change", () => {
    const numCities = readNumCities();
    points = locations(numCities) as Location[];
    clearBest();
    drawPoints(points);
    statusEl.textContent = `Updated problem size to ${numCities} cities.`;
});

async function runSearch(mode: "parallel" | "sequential") {
    const iterations = Math.max(1, Number(iterationsEl.value) || 1);
    const threads = Math.max(1, Number(threadsEl.value) || 1);
    const seedInput = Math.max(1, Number(seedEl.value) || 1);
    const seed = BigInt(Math.trunc(seedInput));
    const numCities = readNumCities();

    if (points.length !== numCities) {
        points = locations(numCities) as Location[];
        clearBest();
        drawPoints(points);
    }

    runParallelEl.disabled = true;
    runSequentialEl.disabled = true;
    resetEl.disabled = true;
    statusEl.textContent = mode === "parallel" ? "Running parallel search..." : "Running sequential search...";

    try {
        if (mode === "parallel" && !initialized) {
            await init_thread_pool(threads);
            initialized = true;
            statusEl.textContent = `Thread pool initialized with ${threads} threads.`;
        }

        const result = mode === "parallel"
            ? (run_best_tour(iterations, seed, threads, numCities) as Result)
            : (run_best_tour_seq(iterations, seed, numCities) as Result);

        const isNewBest = !bestSoFar || result.best_distance < bestSoFar.best_distance;
        if (isNewBest) {
            bestSoFar = result;
            drawTour(points, result.best_tour);
        }

        updateStats(bestSoFar ?? result);

        statusEl.textContent = isNewBest
            ? `${mode === "parallel" ? "Parallel" : "Sequential"} run found a new best tour.`
            : `${mode === "parallel" ? "Parallel" : "Sequential"} run completed; best tour unchanged.`;
    } catch (err) {
        statusEl.textContent = `Error: ${String(err)}`;
    } finally {
        runParallelEl.disabled = false;
        runSequentialEl.disabled = false;
        resetEl.disabled = false;
    }
}

function readNumCities() {
    const parsed = numCitiesEl.valueAsNumber;

    if (!Number.isFinite(parsed)) {
        numCitiesEl.value = String(currentNumCities);
        return currentNumCities;
    }

    const numCities = Math.max(MIN_CITIES, Math.min(MAX_CITIES, Math.trunc(parsed)));
    currentNumCities = numCities;
    numCitiesEl.value = String(numCities);
    return numCities;
}

function clearBest() {
    bestSoFar = null;
    bestDistanceEl.textContent = "-";
    elapsedEl.textContent = "-";
    ipsEl.textContent = "-";
}

function updateStats(result: Result) {
    bestDistanceEl.textContent = result.best_distance.toFixed(3);
    elapsedEl.textContent = `${result.elapsed_ms.toFixed(1)} ms`;
    const ips = result.iterations / Math.max(result.elapsed_ms / 1000, 1e-9);
    ipsEl.textContent = ips.toFixed(0);
}

function drawPoints(locations: Location[]) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
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
        x: pad + ((p.x - minX) / spanX) * (canvas.width - pad * 2),
        y: canvas.height - (pad + ((p.y - minY) / spanY) * (canvas.height - pad * 2))
    }));
}

void setup();
