import { useEffect, useMemo, useRef, useState } from "react";
import { locations } from "../pkg/wasm_bindings.js";
import { CanvasView } from "./components/CanvasView";
import { CodeCard } from "./components/CodeCard";
import { ControlsSection } from "./components/ControlsSection";
import { StatusSection } from "./components/StatusSection";
import { PARALLEL_CODE, PARALLEL_HELP, SEQUENTIAL_CODE, SEQUENTIAL_HELP } from "./code-snippets";
import { runSearchOnce } from "./search-runner";
import type { Location, RunSettings, SearchMode, SearchRequest, SearchResult } from "./shared-types";

const MIN_CITIES = 5;
const MAX_CITIES = 200;
const MAX_THREADS = 16;

function readCssColor(variableName: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(variableName).trim();
}

function clamp(n: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, Math.trunc(n)));
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

function normalizeSeed(seed: number): bigint {
    const parsed = Number.isFinite(seed) ? seed : 1;
    return BigInt(Math.max(1, Math.trunc(parsed)));
}

function generatePoints(seed: bigint, numCities: number) {
    return locations(seed, numCities) as Location[];
}

export function App() {
    const [status, setStatus] = useState("Initializing...");
    const [iterations, setIterations] = useState(10_000);
    const [threads, setThreads] = useState(4);
    const [chunkSize, setChunkSize] = useState(0);
    const [seed, setSeed] = useState(42);
    const [numCities, setNumCities] = useState(50);
    const [points, setPoints] = useState<Location[]>([]);
    const [best, setBest] = useState<SearchResult | null>(null);
    const [bestDistance, setBestDistance] = useState("-");
    const [elapsed, setElapsed] = useState("-");
    const [ips, setIps] = useState("-");
    const [isRunning, setIsRunning] = useState(false);
    const [runMode, setRunMode] = useState<SearchMode>("parallel");
    const [runSubtitle, setRunSubtitle] = useState("Working through candidate tours. Larger runs can take a while.");
    const [runElapsed, setRunElapsed] = useState("Elapsed: 0.0 s");

    const runTicker = useRef<number | undefined>(undefined);
    const runStartedAtMs = useRef(0);

    const cityNodeColor = useMemo(() => readCssColor("--city-node"), []);
    const tourLineColor = useMemo(() => readCssColor("--tour-line"), []);
    const canvasBackgroundColor = useMemo(() => readCssColor("--canvas-bg"), []);

    useEffect(() => {
        const initialPoints = generatePoints(normalizeSeed(seed), numCities);
        setPoints(initialPoints);
        setStatus("Ready");
    }, []);

    useEffect(() => {
        return () => {
            if (runTicker.current !== undefined) {
                window.clearInterval(runTicker.current);
            }
        };
    }, []);

    function clearBest() {
        setBest(null);
        setBestDistance("-");
        setElapsed("-");
        setIps("-");
    }

    function updateStats(result: SearchResult) {
        setBestDistance(result.best_distance.toFixed(3));
        setElapsed(`${result.elapsed_ms.toFixed(1)} ms`);
        const value = result.iterations / Math.max(result.elapsed_ms / 1000, 1e-9);
        setIps(value.toFixed(0));
    }

    function setRunningView(mode: SearchMode, running: boolean) {
        setRunMode(mode);
        setIsRunning(running);

        if (running) {
            runStartedAtMs.current = performance.now();
            setRunSubtitle("Evaluating tours with 2-opt local search. Larger instances can take longer.");
            setRunElapsed("Elapsed: 0.0 s");

            if (runTicker.current !== undefined) {
                window.clearInterval(runTicker.current);
            }

            runTicker.current = window.setInterval(() => {
                const secs = (performance.now() - runStartedAtMs.current) / 1000;
                setRunElapsed(`Elapsed: ${secs.toFixed(1)} s`);
            }, 200);
            return;
        }

        if (runTicker.current !== undefined) {
            window.clearInterval(runTicker.current);
            runTicker.current = undefined;
        }
    }

    function readRunSettings(mode: SearchMode): RunSettings {
        return {
            mode,
            iterations: clamp(iterations, 1, 200_000),
            threads: clamp(threads, 0, MAX_THREADS),
            chunkSize: Math.max(0, Math.trunc(chunkSize)),
            seed: normalizeSeed(seed),
            numCities: clamp(numCities, MIN_CITIES, MAX_CITIES)
        };
    }

    function applyNumCities(next: number) {
        const normalized = clamp(next, MIN_CITIES, MAX_CITIES);
        setNumCities(normalized);
        setPoints(generatePoints(normalizeSeed(seed), normalized));
        clearBest();
        setStatus(`Updated problem size to ${normalized} cities.`);
    }

    function applySeed(next: number) {
        const normalizedSeed = Math.max(1, Math.trunc(Number.isFinite(next) ? next : seed));
        const normalizedCities = clamp(numCities, MIN_CITIES, MAX_CITIES);
        setSeed(normalizedSeed);
        setPoints(generatePoints(normalizeSeed(normalizedSeed), normalizedCities));
        clearBest();
        setStatus(`Updated city seed to ${normalizedSeed}.`);
    }

    async function runSearch(mode: SearchMode) {
        const settings = readRunSettings(mode);

        if (settings.numCities !== numCities) {
            setNumCities(settings.numCities);
        }

        if (settings.seed !== normalizeSeed(seed) || points.length !== settings.numCities) {
            setPoints(generatePoints(settings.seed, settings.numCities));
            clearBest();
        }

        const request: SearchRequest = {
            settings,
            locations: points.length === settings.numCities ? points : generatePoints(settings.seed, settings.numCities)
        };

        setRunningView(settings.mode, true);
        await allowRunningOverlayToRender();
        setStatus(settings.mode === "parallel" ? "Running parallel search..." : "Running sequential search...");

        try {
            const result = await runSearchOnce(request);

            if (!best || result.best_distance < best.best_distance) {
                setBest(result);
            }

            updateStats(result);
            setRunSubtitle(`Processed ${settings.iterations.toLocaleString()} iterations in one call.`);
            setStatus(`${settings.mode === "parallel" ? "Parallel" : "Sequential"} run completed.`);
        } catch (err) {
            setStatus(`Error: ${String(err)}`);
        } finally {
            setRunningView(settings.mode, false);
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
                        Randomly create tours and locally optimize with two-opt <code>Iterations</code> times, and return the best
                        tour.
                    </p>
                </article>
                <article className="intro-card">
                    <h2>Frontend</h2>
                    <p>React + TypeScript + Vite UI.</p>
                </article>
                <article className="intro-card">
                    <CodeCard
                        title="Sequential Code"
                        helpTitle="Sequential Code Breakdown"
                        helpBody={SEQUENTIAL_HELP}
                        code={SEQUENTIAL_CODE}
                    />
                </article>
                <article className="intro-card">
                    <CodeCard
                        title="Parallel Code"
                        helpTitle="Parallel Code Breakdown"
                        helpBody={PARALLEL_HELP}
                        code={PARALLEL_CODE}
                    />
                </article>
            </section>

            <ControlsSection
                iterations={iterations}
                threads={threads}
                chunkSize={chunkSize}
                seed={seed}
                numCities={numCities}
                isRunning={isRunning}
                status={status}
                onIterationsChange={(next) => setIterations(clamp(next, 1, 200_000))}
                onThreadsChange={(next) => {
                    const normalized = clamp(next, 0, MAX_THREADS);
                    setThreads(normalized);
                    setStatus(`Thread limit set to ${normalized}.`);
                }}
                onChunkSizeChange={(next) => {
                    const normalized = Math.max(0, Math.trunc(next));
                    setChunkSize(normalized);
                    setStatus(`Chunk size set to ${normalized}.`);
                }}
                onSeedChange={applySeed}
                onNumCitiesChange={applyNumCities}
                onRunParallel={() => {
                    void runSearch("parallel");
                }}
                onRunSequential={() => {
                    void runSearch("sequential");
                }}
                onReset={() => {
                    clearBest();
                    setPoints(generatePoints(normalizeSeed(seed), numCities));
                    setStatus("Best tour reset. Ready for a fresh run.");
                }}
            />

            <section className="card">
                <StatusSection
                    isRunning={isRunning}
                    runMode={runMode}
                    runSubtitle={runSubtitle}
                    runElapsed={runElapsed}
                    bestDistance={bestDistance}
                    elapsed={elapsed}
                    ips={ips}
                />
                <CanvasView
                    points={points}
                    best={best}
                    cityNodeColor={cityNodeColor}
                    tourLineColor={tourLineColor}
                    canvasBackgroundColor={canvasBackgroundColor}
                />
            </section>
        </main>
    );
}
