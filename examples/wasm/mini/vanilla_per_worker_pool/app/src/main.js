const inputEl = document.getElementById("input");
const threadsEl = document.getElementById("num_threads");
const runEl = document.getElementById("run");
const resultEl = document.getElementById("result");

runEl.addEventListener("click", () => {
    const input = Number(inputEl.value);
    const threads = Number(threadsEl.value);
    resultEl.textContent = '-'

    const worker = new Worker(new URL("./worker.js", import.meta.url), {
        type: "module"
    });

    worker.addEventListener(
        "message",
        (event) => {
            const data = event.data;

            if (data.type === "ok") {
                resultEl.textContent = String(data.result);
            } else {
                resultEl.textContent = `error: ${data.message}`;
            }

            worker.terminate();
        },
        { once: true }
    );

    worker.postMessage({ input, threads });
});
