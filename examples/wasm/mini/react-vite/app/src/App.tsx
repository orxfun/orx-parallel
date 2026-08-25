import { useEffect, useState } from "react";
import type { ParallelWorker } from "orx-parallel-wasm";

type Computations = {
    calculate_fibonacci: (workload: number, threads: number) => bigint;
    mandelbrot_checksum: (limit: number, threads: number) => number;
};

type AppProps = {
    worker: ParallelWorker<Computations>;
};

function readPositive(value: string): number {
    const parsed = Number.parseInt(value, 10);
    return Number.isFinite(parsed) ? Math.max(1, parsed) : 1;
}

export function App({ worker }: AppProps) {
    const [threads, setThreads] = useState(0);
    const [maxThreads, setMaxThreads] = useState(1);
    const [poolStatus, setPoolStatus] = useState("Initializing thread pool...");
    const [fibonacciWorkload, setFibonacciWorkload] = useState("50000");
    const [mandelbrotWorkload, setMandelbrotWorkload] = useState("50000");
    const [fibonacciResult, setFibonacciResult] = useState("No result yet.");
    const [mandelbrotResult, setMandelbrotResult] = useState("No result yet.");
    const [running, setRunning] = useState<string | null>(null);

    useEffect(() => {
        void worker.ready().then(
            () => {
                const initializedThreads = worker.initializedThreads ?? 1;
                setMaxThreads(initializedThreads);
                setThreads(0);
                setPoolStatus(`Thread pool ready: ${initializedThreads} threads`);
            },
            (error: unknown) => {
                setPoolStatus(`Thread pool error: ${error instanceof Error ? error.message : String(error)}`);
            }
        );
    }, [worker]);

    async function run(
        name: string,
        computation: () => Promise<bigint | number>,
        setResult: (result: string) => void
    ) {
        setRunning(name);
        setResult("Running...");
        const startedAt = performance.now();

        try {
            const result = await computation();
            setResult(`Result: ${String(result)} | ${(performance.now() - startedAt).toFixed(2)} ms`);
        } catch (error) {
            setResult(`Error: ${error instanceof Error ? error.message : String(error)}`);
        } finally {
            setRunning(null);
        }
    }

    const selectedThreads = Number.isFinite(threads) ? Math.max(0, Math.min(maxThreads, threads)) : 0;

    return (
        <main>
            <p className="eyebrow">orx-parallel / WebAssembly</p>
            <h1>Parallel computations using one shared thread pool</h1>
            <p className="intro">Run <code>orx-parallel</code> computations with different worker counts.</p>

            <section className="panel" aria-labelledby="settings-title">
                <h2 id="settings-title">Run settings</h2>
                <label>
                    Threads
                    <input
                        id="threads"
                        type="number"
                        value={selectedThreads}
                        min="0"
                        max={maxThreads}
                        step="1"
                        onChange={(event) => setThreads(Number.parseInt(event.target.value, 10) || 0)}
                    />
                    <span id="threads-help">0 uses all {maxThreads} initialized threads</span>
                </label>
                <p id="pool-status" role="status">{poolStatus}</p>
            </section>

            <section className="computations" aria-label="Computations">
                <article className="computation">
                    <p className="index">01</p>
                    <h2>Fibonacci workload</h2>
                    <p>Sum many Fibonacci terms to give each worker useful CPU work.</p>
                    <label>
                        Number of terms
                        <input
                            id="fibonacci-workload"
                            type="number"
                            value={fibonacciWorkload}
                            min="1"
                            step="1000"
                            onChange={(event) => setFibonacciWorkload(event.target.value)}
                        />
                    </label>
                    <button
                        id="run-fibonacci"
                        type="button"
                        disabled={running !== null || poolStatus.startsWith("Thread pool error")}
                        onClick={() => void run("fibonacci", () => worker.call("calculate_fibonacci", [readPositive(fibonacciWorkload), selectedThreads]), setFibonacciResult)}
                    >
                        Calculate Fibonacci
                    </button>
                    <p id="fibonacci-result" className="result">{fibonacciResult}</p>
                </article>

                <article className="computation">
                    <p className="index">02</p>
                    <h2>Mandelbrot checksum</h2>
                    <p>Calculate a checksum across a configurable number of Mandelbrot points.</p>
                    <label>
                        Number of points
                        <input
                            id="mandelbrot-workload"
                            type="number"
                            value={mandelbrotWorkload}
                            min="1"
                            step="1000"
                            onChange={(event) => setMandelbrotWorkload(event.target.value)}
                        />
                    </label>
                    <button
                        id="run-mandelbrot"
                        type="button"
                        disabled={running !== null || poolStatus.startsWith("Thread pool error")}
                        onClick={() => void run("mandelbrot", () => worker.call("mandelbrot_checksum", [readPositive(mandelbrotWorkload), selectedThreads]), setMandelbrotResult)}
                    >
                        Calculate Checksum
                    </button>
                    <p id="mandelbrot-result" className="result">{mandelbrotResult}</p>
                </article>
            </section>
        </main>
    );
}