import { ParallelWorker } from "orx-parallel-web";
import bindingsUrl from "../pkg/wasm_bindings.js?url";

type Computations = {
    compute: (input: number, threads: number) => bigint;
};

const inputEl = document.getElementById("input") as HTMLInputElement;
const threadsEl = document.getElementById("num_threads") as HTMLInputElement;
const runEl = document.getElementById("run") as HTMLButtonElement;
const resultEl = document.getElementById("result") as HTMLParagraphElement;

const worker = new ParallelWorker<Computations>({
    bindingsUrl,
    methods: ["compute"],
    threads: 0
});

runEl.addEventListener("click", async () => {
    const input = Number(inputEl.value);
    const threads = Number(threadsEl.value);
    resultEl.textContent = "running...";

    try {
        const result = await worker.call("compute", [input, threads]);
        resultEl.textContent = String(result);
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        resultEl.textContent = `error: ${message}`;
    }
});

window.addEventListener("beforeunload", () => worker.terminate());
