const inputEl = document.getElementById("input");
const threadsEl = document.getElementById("num_threads");
const runEl = document.getElementById("run");
const resultEl = document.getElementById("result");

const threads = Number(threadsEl.value);
const worker = new Worker(new URL("./worker.js", import.meta.url), {
    type: "module"
});

worker.postMessage({ type: "init", threads });

worker.addEventListener("message", (event) => {
    const data = event.data;

    if (data.type === "ok") {
        resultEl.textContent = String(data.result);
    } else if (data.type === "err") {
        resultEl.textContent = `error: ${data.message}`;
    }
});

runEl.addEventListener("click", () => {
    const input = Number(inputEl.value);
    resultEl.textContent = '-';
    worker.postMessage({ type: "compute", input });
});
