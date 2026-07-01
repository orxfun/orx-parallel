import init, {
    init_parallel_runtime,
    run_fib_sum
} from "../pkg/orx_parallel_wasm_vite_demo_basic.js";

type FibSumResult = {
    start: number;
    end: number;
    sum: number;
    threads: number;
    elapsed_ms: number;
};

const startEl = document.getElementById("start") as HTMLInputElement;
const endEl = document.getElementById("end") as HTMLInputElement;
const runEl = document.getElementById("run") as HTMLButtonElement;
const statusEl = document.getElementById("status") as HTMLParagraphElement;
const outputEl = document.getElementById("output") as HTMLPreElement;

let runtimeReady = false;

async function setup() {
    await init();
    statusEl.textContent = "Ready.";
}

runEl.addEventListener("click", async () => {
    const start = Math.max(0, Math.trunc(Number(startEl.value) || 0));
    const end = Math.max(0, Math.trunc(Number(endEl.value) || 0));

    runEl.disabled = true;
    statusEl.textContent = "Running...";
    outputEl.textContent = "";

    try {
        if (!runtimeReady) {
            await init_parallel_runtime();
            runtimeReady = true;
        }

        const result = run_fib_sum(start, end) as FibSumResult;
        outputEl.textContent = `Computed in parallel with ${result.threads} threads, the sum of Fibonacci numbers from ${result.start} to ${result.end} is ${result.sum}.`;
        statusEl.textContent = `Done in ${result.elapsed_ms.toFixed(1)} ms.`;
    } catch (err) {
        statusEl.textContent = `Error: ${String(err)}`;
    } finally {
        runEl.disabled = false;
    }
});

void setup();
