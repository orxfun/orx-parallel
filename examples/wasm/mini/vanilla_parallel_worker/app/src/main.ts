import { ParallelWorker } from "orx-parallel-web";
import bindingsUrl from "../pkg/wasm_bindings.js?url";

type Computations = {
    compute: (input: number, threads: number) => bigint;
    compute_chunks: (input: number, threads: number, chunkSize: number) => bigint;
};

const inputEl = document.getElementById("input") as HTMLInputElement;
const threadsEl = document.getElementById("num_threads") as HTMLInputElement;
const chunkSizeEl = document.getElementById("chunk_size") as HTMLInputElement;
const runEl = document.getElementById("run") as HTMLButtonElement;
const runChunksEl = document.getElementById("run_chunks") as HTMLButtonElement;
const resultEl = document.getElementById("result") as HTMLParagraphElement;
const durationEl = document.getElementById("duration") as HTMLParagraphElement;
const poolThreadsEl = document.getElementById("pool_threads") as HTMLParagraphElement;

const worker = new ParallelWorker<Computations>({
    bindingsUrl,
    methods: ["compute", "compute_chunks"],
    threads: 0
});

void worker.ready().then(
    () => poolThreadsEl.textContent = `thread pool initialized with ${worker.initializedThreads} threads`,
    (error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        poolThreadsEl.textContent = `thread pool error: ${message}`;
    }
);

async function run(computation: () => Promise<bigint>): Promise<void> {
    runEl.disabled = true;
    runChunksEl.disabled = true;
    resultEl.textContent = "running...";
    durationEl.textContent = "-";
    const start = performance.now();

    try {
        const result = await computation();
        const durationMillis = performance.now() - start;
        resultEl.textContent = String(result);
        durationEl.textContent = `completed in ${durationMillis.toFixed(3)} milliseconds`;
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        resultEl.textContent = `error: ${message}`;
    } finally {
        runEl.disabled = false;
        runChunksEl.disabled = false;
    }
}

runEl.addEventListener("click", () =>
    run(() => worker.call("compute", [Number(inputEl.value), Number(threadsEl.value)]))
);

runChunksEl.addEventListener("click", () =>
    run(() =>
        worker.call("compute_chunks", [
            Number(inputEl.value),
            Number(threadsEl.value),
            Number(chunkSizeEl.value)
        ])
    )
);

window.addEventListener("beforeunload", () => worker.terminate());
