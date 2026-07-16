import { useEffect, useRef, useState } from "react";
import init from "../pkg/orx_parallel_wasm_tsp_react.js";
import { locations } from "../pkg/orx_parallel_wasm_tsp_react.js";
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

const MIN_CITIES = 5;
const MAX_CITIES = 200;
const MIN_THREADS = 1;
const MAX_THREADS = 16;

const SEQUENTIAL_CODE = `let mut rng = SmallRng::seed_from_u64(seed);
(0..iterations)
    .map(|_| create_tour(&mut rng, locations))
    .min_by_key(|x| OrderedFloat::from(x.distance))`;

const PARALLEL_CODE = `(0..iterations)
    .into_par()
    .use_new(|t| SmallRng::seed_from_u64(seed + t as u64))
    .map(|rng, _| create_tour(rng, locations))
    .min_by_key(|x| OrderedFloat::from(x.distance))`;

const SEQUENTIAL_HELP = `// random-number-generator to construct initial random tours
let mut rng = SmallRng::seed_from_u64(seed);

// we will construct & improve \`iterations\` tours
(0..iterations)

    // \`create_tour\` constructs a random tour and locally optimizes within 2-opt
    .map(|_| create_tour(&mut rng, locations))

    // among all created tours, we pick the one with minimum distance
    .min_by_key(|x| OrderedFloat::from(x.distance))`;

const PARALLEL_HELP = `// we will construct & improve \`iterations\` tours
(0..iterations)

    // convert the iterator into parallel iterator
    .into_par()

    // \`use_new\` enables mutable variables in parallel computations
    // each thread will have its own random number generator
    .use_new(|t| SmallRng::seed_from_u64(seed + t as u64))

    // \`create_tour\` constructs a random tour and locally optimizes within 2-opt
    .map(|rng, _| create_tour(rng, locations))

    // among all created tours, we pick the one with minimum distance
    .min_by_key(|x| OrderedFloat::from(x.distance))`;

export default function App() {
    const canvasRef = useRef<HTMLCanvasElement | null>(null);
    const runTickerRef = useRef<number | undefined>(undefined);
    const runStartedAtRef = useRef(0);

    const [status, setStatus] = useState("Initializing...");
    const [iterations, setIterations] = useState(10000);
    const [threads, setThreads] = useState(4);
    const [chunkSize, setChunkSize] = useState(0);
    const [seed, setSeed] = useState(42);
    const [numCities, setNumCities] = useState(50);
    const [points, setPoints] = useState<Location[]>([]);
    const [bestSoFar, setBestSoFar] = useState<RunAggregate | null>(null);
    const [bestDistance, setBestDistance] = useState("-");
    const [elapsed, setElapsed] = useState("-");
    const [ips, setIps] = useState("-");
    const [isRunning, setIsRunning] = useState(false);
    const [runMode, setRunMode] = useState<SearchMode>("parallel");
    const [runSubtitle, setRunSubtitle] = useState(
        "Working through candidate tours. Larger runs can take a while."
    );

    useEffect(() => {
        let cancelled = false;

        void (async () => {
            try {
                await init();
                if (cancelled) {
                    return;
                }

                const initialPoints = generatePoints(toSeed(seed), numCities);
                setPoints(initialPoints);
                setStatus("Ready");
                highlightCodeBlocks();
            } catch (err) {
                if (!cancelled) {
                    setStatus(`Failed to initialize wasm: ${String(err)}`);
                }
            }
        })();

        return () => {
            cancelled = true;
            if (runTickerRef.current !== undefined) {
                window.clearInterval(runTickerRef.current);
            }
        };
    }, []);

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) {
            return;
        }

        const ctx = canvas.getContext("2d");
        if (!ctx) {
            return;
        }

        drawPoints(ctx, canvas, points);
        if (bestSoFar) {
            drawTour(ctx, canvas, points, bestSoFar.best_tour);
        }
    }, [points, bestSoFar]);

    function clearBest() {
        setBestSoFar(null);
        setBestDistance("-");
        setElapsed("-");
        setIps("-");
    }

    function updateNumCities(nextValue: number) {
        const nextNumCities = clamp(nextValue, MIN_CITIES, MAX_CITIES);
        setNumCities(nextNumCities);
        setPoints(generatePoints(toSeed(seed), nextNumCities));
        clearBest();
        setStatus(`Updated problem size to ${nextNumCities} cities.`);
    }

    function updateSeed(nextValue: number) {
        const nextSeed = Math.max(1, Math.trunc(nextValue) || 1);
        setSeed(nextSeed);
        setPoints(generatePoints(toSeed(nextSeed), numCities));
        clearBest();
        setStatus(`Updated city seed to ${nextSeed.toString()}.`);
    }

    function updateThreads(nextValue: number) {
        const nextThreads = clamp(nextValue, MIN_THREADS, MAX_THREADS);
        setThreads(nextThreads);
        setStatus(`Thread limit set to ${nextThreads}.`);
    }

    function updateChunkSize(nextValue: number) {
        const nextChunkSize = Math.max(0, Math.trunc(nextValue) || 0);
        setChunkSize(nextChunkSize);
        setStatus(`Chunk size set to ${nextChunkSize}.`);
    }

    async function runSearch(mode: SearchMode) {
        const settings: RunSettings = {
            mode,
            iterations: clamp(iterations, 1, Number.MAX_SAFE_INTEGER),
            threads: clamp(threads, MIN_THREADS, MAX_THREADS),
            chunkSize: Math.max(0, Math.trunc(chunkSize)),
            seed: toSeed(seed),
            numCities: clamp(numCities, MIN_CITIES, MAX_CITIES)
        };

        const currentPoints = points.length === settings.numCities ? points : generatePoints(settings.seed, settings.numCities);
        if (currentPoints !== points) {
            setPoints(currentPoints);
        }

        setRunMode(mode);
        setIsRunning(true);
        setRunSubtitle("Evaluating tours with 2-opt local search. Larger instances can take longer.");
        setStatus(mode === "parallel" ? "Running parallel search..." : "Running sequential search...");
        runStartedAtRef.current = performance.now();

        if (runTickerRef.current !== undefined) {
            window.clearInterval(runTickerRef.current);
        }

        runTickerRef.current = window.setInterval(() => {
            const secs = (performance.now() - runStartedAtRef.current) / 1000;
            setElapsed(`${secs.toFixed(1)} s`);
        }, 200);

        await nextPaint();
        await nextPaint();
        await delay(24);

        try {
            const result = await runSearchOnce(settings);

            if (!bestSoFar || result.best_distance < bestSoFar.best_distance) {
                setBestSoFar({
                    best_tour: result.best_tour,
                    best_distance: result.best_distance,
                    iterations: result.iterations,
                    elapsed_ms: result.elapsed_ms
                });
            }

            setBestDistance(result.best_distance.toFixed(3));
            setElapsed(`${result.elapsed_ms.toFixed(1)} ms`);
            const iterationsPerSecond = result.iterations / Math.max(result.elapsed_ms / 1000, 1e-9);
            setIps(iterationsPerSecond.toFixed(0));
            setRunSubtitle(`Processed ${settings.iterations.toLocaleString()} iterations in one call.`);
            setStatus(`${mode === "parallel" ? "Parallel" : "Sequential"} run completed.`);
        } catch (err) {
            setStatus(`Error: ${String(err)}`);
        } finally {
            setIsRunning(false);
            if (runTickerRef.current !== undefined) {
                window.clearInterval(runTickerRef.current);
                runTickerRef.current = undefined;
            }
        }
    }

    return (
        <main>
            <header className="hero">
                <a href="https://github.com/orxfun" target="_blank" rel="noreferrer">
                    <img
                        className="hero-logo"
                        src="https://avatars.githubusercontent.com/u/132661625?s=400&u=e13dbda1a79636fa7d02dd9ac8dfc02705694144&v=4"
                        alt="orx logo"
                    />
                </a>
                <div className="hero-copy">
                    <h1>Parallel computation in WASM</h1>
                    <p className="hero-kicker">A React TSP demo with shared-memory wasm threads using orx-parallel.</p>
                </div>
            </header>

            <section className="intro-cards" aria-label="Example overview">
                <article className="intro-card">
                    <h2>Computation: Local Search for TSP</h2>
                    <p>
                        Randomly create tours and locally optimize with two-opt <code>Iterations</code> times, and
                        return the best tour.
                    </p>
                </article>
                <article className="intro-card">
                    <h2>Frontend</h2>
                    <p>React + Vite + TypeScript UI.</p>
                </article>
                <article className="intro-card">
                    <CodeCard title="Sequential Code" helpTitle="Sequential Code Breakdown" helpBody={SEQUENTIAL_HELP} code={SEQUENTIAL_CODE} />
                </article>
                <article className="intro-card">
                    <CodeCard title="Parallel Code" helpTitle="Parallel Code Breakdown" helpBody={PARALLEL_HELP} code={PARALLEL_CODE} />
                </article>
            </section>

            <section className="card">
                <div className={`run-overlay${isRunning ? " active" : ""}`} aria-live="polite" aria-hidden={!isRunning}>
                    <div className="run-overlay-card">
                        <div className="run-overlay-top">
                            <span className="spinner" aria-hidden="true"></span>
                            <p id="runTitle" className="run-title">
                                {runMode === "parallel" ? "Running parallel search..." : "Running sequential search..."}
                            </p>
                        </div>
                        <p className="run-subtitle">{runSubtitle}</p>
                        <p className="run-elapsed">Elapsed: {isRunning ? `${((performance.now() - runStartedAtRef.current) / 1000).toFixed(1)} s` : elapsed}</p>
                        <div className="run-bar" aria-hidden="true"></div>
                    </div>
                </div>

                <div className="control-panel">
                    <div className="controls">
                        <label>
                            Number of cities
                            <input id="numCities" type="number" min="5" max="200" value={numCities} onChange={(event) => updateNumCities(event.currentTarget.valueAsNumber)} />
                        </label>
                        <label>
                            Iterations
                            <input id="iterations" type="number" min="1" max="200000" value={iterations} onChange={(event) => setIterations(clamp(event.currentTarget.valueAsNumber, 1, 200000))} />
                        </label>
                        <label>
                            Threads (1..16)
                            <input id="threads" type="number" min="1" max="16" value={threads} onChange={(event) => updateThreads(event.currentTarget.valueAsNumber)} />
                        </label>
                        <label>
                            Chunk size
                            <input id="chunkSize" type="number" min="0" max="1048576" value={chunkSize} onChange={(event) => updateChunkSize(event.currentTarget.valueAsNumber)} />
                        </label>
                        <label>
                            Seed
                            <input id="seed" type="number" min="1" max="99999999" value={seed} onChange={(event) => updateSeed(event.currentTarget.valueAsNumber)} />
                        </label>
                    </div>

                    <div className="actions">
                        <button id="runParallel" onClick={() => void runSearch("parallel")} disabled={isRunning}>
                            Run parallel
                        </button>
                        <button id="runSequential" onClick={() => void runSearch("sequential")} disabled={isRunning}>
                            Run sequential
                        </button>
                        <button
                            id="reset"
                            onClick={() => {
                                clearBest();
                                setPoints(generatePoints(toSeed(seed), numCities));
                                setStatus("Best tour reset. Ready for a fresh run.");
                            }}
                            disabled={isRunning}
                        >
                            Reset
                        </button>
                    </div>

                    <div className="status-value" aria-live="polite">{status}</div>
                </div>

                <div className="stats">
                    <div className="stat">
                        <h3>Best Distance</h3>
                        <p>{bestDistance}</p>
                    </div>
                    <div className="stat">
                        <h3>Elapsed</h3>
                        <p>{elapsed}</p>
                    </div>
                    <div className="stat">
                        <h3>Iterations/s</h3>
                        <p>{ips}</p>
                    </div>
                </div>

                <canvas ref={canvasRef} id="canvas" width="920" height="430"></canvas>
            </section>
        </main>
    );
}

function CodeCard({ title, helpTitle, helpBody, code }: { title: string; helpTitle: string; helpBody: string; code: string }) {
    return (
        <>
            <div className="code-card-header">
                <h2>{title}</h2>
                <details className="code-help">
                    <summary className="code-help-trigger" aria-label={`Show ${title.toLowerCase()} explanation`}>
                        ?
                    </summary>
                    <div className="code-help-popover" role="note">
                        <h2 style={{ paddingLeft: 10 }}>{helpTitle}</h2>
                        <pre className="code-block">
                            <code className="language-rust">{helpBody}</code>
                        </pre>
                    </div>
                </details>
            </div>
            <pre className="code-block">
                <code className="language-rust">{code}</code>
            </pre>
        </>
    );
}

function highlightCodeBlocks() {
    document.querySelectorAll<HTMLElement>(".code-block code").forEach((block) => {
        block.classList.add("language-rust");
        hljs.highlightElement(block);
    });
}

function runSearchOnce(settings: RunSettings): Promise<SearchResult> {
    return new Promise<SearchResult>((resolve, reject) => {
        const worker = new Worker(new URL("./search-worker.ts", import.meta.url), {
            type: "module"
        });

        const cleanup = () => {
            worker.terminate();
        };

        worker.addEventListener(
            "message",
            (event: MessageEvent) => {
                const data = event.data as
                    | { type: "search-result"; result: SearchResult }
                    | { type: "search-error"; message: string };

                if (data.type === "search-error") {
                    cleanup();
                    reject(new Error(data.message));
                    return;
                }

                cleanup();
                resolve(data.result);
            },
            { once: true }
        );

        worker.addEventListener(
            "error",
            (event) => {
                cleanup();
                reject(new Error(event.message || "search worker failed"));
            },
            { once: true }
        );

        worker.postMessage({ type: "run-search", settings });
    });
}

function toSeed(value: number) {
    return BigInt(Math.max(1, Math.trunc(value) || 1));
}

function generatePoints(seed: bigint, numCities: number) {
    return locations(seed, numCities) as Location[];
}

function drawPoints(ctx: CanvasRenderingContext2D, canvas: HTMLCanvasElement, locations: Location[]) {
    const palette = readPalette();
    ctx.fillStyle = palette.canvasBackground;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    const mapped = mapPoints(canvas, locations);
    for (const p of mapped) {
        ctx.beginPath();
        ctx.fillStyle = palette.cityNode;
        ctx.arc(p.x, p.y, 6, 0, Math.PI * 2);
        ctx.fill();
    }
}

function drawTour(ctx: CanvasRenderingContext2D, canvas: HTMLCanvasElement, locations: Location[], tour: number[]) {
    drawPoints(ctx, canvas, locations);
    if (tour.length === 0) {
        return;
    }

    const palette = readPalette();
    const mapped = mapPoints(canvas, locations);
    ctx.beginPath();
    ctx.strokeStyle = palette.tourLine;
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

function mapPoints(canvas: HTMLCanvasElement, locations: Location[]) {
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

function readPalette() {
    const styles = getComputedStyle(document.documentElement);
    return {
        cityNode: styles.getPropertyValue("--city-node").trim() || "#f59e0b",
        tourLine: styles.getPropertyValue("--tour-line").trim() || "#1d4ed8",
        canvasBackground: styles.getPropertyValue("--code-block-bg").trim() || "#0f172a"
    };
}

function clamp(value: number, min: number, max: number) {
    if (!Number.isFinite(value)) {
        return min;
    }

    return Math.max(min, Math.min(max, Math.trunc(value)));
}

function nextPaint() {
    return new Promise<void>((resolve) => {
        requestAnimationFrame(() => resolve());
    });
}

function delay(ms: number) {
    return new Promise<void>((resolve) => window.setTimeout(resolve, ms));
}