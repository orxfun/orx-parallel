import { ParallelWorker } from "orx-parallel-wasm";
import bindingsUrl from "../pkg/wasm_bindings.js?url";
import "../style.css";

type Computations = {
    calculate_fibonacci: (workload: number, threads: number) => bigint;
    count_primes: (limit: number, threads: number) => number;
};

const ui = {
    threads: document.querySelector<HTMLInputElement>("#threads")!,
    threadsHelp: document.querySelector<HTMLSpanElement>("#threads-help")!,
    poolStatus: document.querySelector<HTMLParagraphElement>("#pool-status")!,
    fibonacciWorkload: document.querySelector<HTMLInputElement>("#fibonacci-workload")!,
    primeLimit: document.querySelector<HTMLInputElement>("#prime-limit")!,
    runFibonacci: document.querySelector<HTMLButtonElement>("#run-fibonacci")!,
    runPrimes: document.querySelector<HTMLButtonElement>("#run-primes")!,
    fibonacciResult: document.querySelector<HTMLParagraphElement>("#fibonacci-result")!,
    primeResult: document.querySelector<HTMLParagraphElement>("#prime-result")!
};

const worker = new ParallelWorker<Computations>({
    bindingsUrl,
    methods: ["calculate_fibonacci", "count_primes"],
    threads: 0
});

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
        ui.runPrimes.disabled = true;
    }
);

ui.runFibonacci.addEventListener("click", () => {
    void run(ui.runFibonacci, ui.fibonacciResult, () =>
        worker.call("calculate_fibonacci", [readPositive(ui.fibonacciWorkload), readThreads()])
    );
});

ui.runPrimes.addEventListener("click", () => {
    void run(ui.runPrimes, ui.primeResult, () =>
        worker.call("count_primes", [readPositive(ui.primeLimit), readThreads()])
    );
});

window.addEventListener("beforeunload", () => worker.terminate());
