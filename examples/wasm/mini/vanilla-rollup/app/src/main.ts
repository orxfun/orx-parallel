import { ParallelWorker } from "orx-parallel-wasm";

// orx-parallel-wasm/rollup always emits this canonical entry, regardless of crate name.
const bindingsUrl = new URL("assets/bindings.js", document.baseURI);

// Desired number of threads in the thread pool, if the hardware allows.
// Setting it to 0 allows using all available threads.
const THREADS_IN_POOL = 0;

type Computations = {
    calculate_fibonacci: (workload: number, threads: number) => bigint;
    mandelbrot_checksum: (limit: number, threads: number) => number;
};

// Create worker with exported parallel, or sequential, computations
const worker = new ParallelWorker<Computations>({
    bindingsUrl,
    methods: ["calculate_fibonacci", "mandelbrot_checksum"],
    threads: THREADS_IN_POOL
});

const ui = {
    threads: document.querySelector<HTMLInputElement>("#threads")!,
    threadsHelp: document.querySelector<HTMLSpanElement>("#threads-help")!,
    poolStatus: document.querySelector<HTMLParagraphElement>("#pool-status")!,
    fibonacciWorkload: document.querySelector<HTMLInputElement>("#fibonacci-workload")!,
    mandelbrotWorkload: document.querySelector<HTMLInputElement>("#mandelbrot-workload")!,
    runFibonacci: document.querySelector<HTMLButtonElement>("#run-fibonacci")!,
    runMandelbrot: document.querySelector<HTMLButtonElement>("#run-mandelbrot")!,
    fibonacciResult: document.querySelector<HTMLParagraphElement>("#fibonacci-result")!,
    mandelbrotResult: document.querySelector<HTMLParagraphElement>("#mandelbrot-result")!
};

// Per-computation thread limit.
// Setting it to 0 allows using all threads in the thread pool.
function readThreads(): number {
    const value = Number.parseInt(ui.threads.value, 10);
    const maxThreads = worker.initializedThreads ?? 1;
    const threads = Number.isFinite(value) ? Math.max(0, Math.min(maxThreads, value)) : 0;
    ui.threads.value = String(threads);
    return threads;
}

function readPositive(input: HTMLInputElement): number {
    const value = Number.parseInt(input.value, 10);
    return Number.isFinite(value) ? Math.max(1, value) : 1;
}

async function run<T>(button: HTMLButtonElement, output: HTMLParagraphElement, computation: () => Promise<T>): Promise<void> {
    button.disabled = true;
    output.textContent = "Running...";
    const startedAt = performance.now();

    try {
        const result = await computation();
        const elapsed = performance.now() - startedAt;
        output.textContent = `Result: ${String(result)} | ${elapsed.toFixed(2)} ms`;
    } catch (error) {
        output.textContent = `Error: ${error instanceof Error ? error.message : String(error)}`;
    } finally {
        button.disabled = false;
    }
}

void worker.ready().then(
    () => {
        ui.threads.max = String(worker.initializedThreads);
        ui.threadsHelp.textContent = `0 uses all ${worker.initializedThreads} initialized threads`;
        ui.poolStatus.textContent = `Thread pool ready: ${worker.initializedThreads} threads`;
    },
    (error: unknown) => {
        ui.poolStatus.textContent = `Thread pool error: ${error instanceof Error ? error.message : String(error)}`;
        ui.runFibonacci.disabled = true;
        ui.runMandelbrot.disabled = true;
    }
);

ui.runFibonacci.addEventListener("click", () => {
    void run(ui.runFibonacci, ui.fibonacciResult, () =>
        worker.call("calculate_fibonacci", [readPositive(ui.fibonacciWorkload), readThreads()])
    );
});

ui.runMandelbrot.addEventListener("click", () => {
    void run(ui.runMandelbrot, ui.mandelbrotResult, () =>
        worker.call("mandelbrot_checksum", [readPositive(ui.mandelbrotWorkload), readThreads()])
    );
});

window.addEventListener("beforeunload", () => worker.terminate());
