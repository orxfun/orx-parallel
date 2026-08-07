const inputEl = document.getElementById("input");
const threadsEl = document.getElementById("num_threads");
const runEl = document.getElementById("run");
const resultEl = document.getElementById("result");

// Set to `N > 0` to limit all computations by `worker` to at most `N` threads.
// Set to `0` to create the pool with all available threads.
const threadsInPool = 32;

const worker = new Worker(new URL("./worker.js", import.meta.url), {
    type: "module"
});

worker.postMessage({ type: "init", threads: threadsInPool });

worker.addEventListener("message", (event) => {
    const data = event.data;

    if (data.type === "ok") {
        resultEl.textContent = String(data.result);
    } else if (data.type === "err") {
        resultEl.textContent = `error: ${data.message}`;
    }
});

runEl.addEventListener("click", () => {
    // this particular computation will use:
    // * `min(threadsInPool, threadsForComputation)` threads when `threadsForComputation > 0`,
    // * `threadsInPool` threads when `threadsForComputation == 0`.
    const threadsForComputation = Number(threadsEl.value);
    const input = Number(inputEl.value);
    resultEl.textContent = '-';
    worker.postMessage({ type: "compute", input, threadsForComputation });
});
