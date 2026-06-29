import { useState } from "react";
import { createRoot } from "react-dom/client";
import init, {
    init_parallel_runtime,
    run_fib_sum
} from "../pkg/orx_parallel_wasm_react_demo_basic.js";

type FibSumResult = {
    start: number;
    end: number;
    sum: number;
    threads: number;
    elapsed_ms: number;
};

let runtimeReady = false;

function App() {
    const [start, setStart] = useState(1);
    const [end, setEnd] = useState(40);
    const [status, setStatus] = useState("Initializing...");
    const [output, setOutput] = useState("");
    const [running, setRunning] = useState(false);

    async function run() {
        const safeStart = Math.max(0, Math.trunc(Number(start) || 0));
        const safeEnd = Math.max(0, Math.trunc(Number(end) || 0));

        setRunning(true);
        setStatus("Running...");
        setOutput("");

        try {
            if (!runtimeReady) {
                await init_parallel_runtime();
                runtimeReady = true;
            }

            const result = run_fib_sum(safeStart, safeEnd) as FibSumResult;
            setOutput(
                `Computed in parallel with ${result.threads} threads, the sum of Fibonacci numbers from ${result.start} to ${result.end} is ${result.sum}.`
            );
            setStatus(`Done in ${result.elapsed_ms.toFixed(1)} ms.`);
        } catch (err) {
            setStatus(`Error: ${String(err)}`);
        } finally {
            setRunning(false);
        }
    }

    return (
        <main>
            <h1>orx-parallel wasm react demo basic</h1>
            <p>Compute the sum of Fibonacci numbers over a range in parallel using 4 threads.</p>

            <label>
                Start:
                <input
                    type="number"
                    min={0}
                    max={93}
                    value={start}
                    onChange={(e) => setStart(Number(e.currentTarget.value))}
                />
            </label>

            <label>
                End:
                <input
                    type="number"
                    min={0}
                    max={93}
                    value={end}
                    onChange={(e) => setEnd(Number(e.currentTarget.value))}
                />
            </label>

            <div>
                <button type="button" disabled={running} onClick={run}>
                    Run parallel Fibonacci sum
                </button>
            </div>

            <p>{status}</p>
            <pre>{output}</pre>
        </main>
    );
}

async function setup() {
    await init();
    const rootEl = document.getElementById("root");
    if (!rootEl) {
        throw new Error("missing root element");
    }
    createRoot(rootEl).render(<App />);
}

void setup();
