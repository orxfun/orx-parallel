import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { ParallelWorker } from "orx-parallel-wasm";
import "../style.css";
import { App } from "./App";

type Computations = {
    calculate_fibonacci: (workload: number, threads: number) => bigint;
    mandelbrot_checksum: (limit: number, threads: number) => number;
};

const worker = new ParallelWorker<Computations>({
    bindingsUrl: new URL("../pkg/wasm_bindings.js", import.meta.url).href,
    methods: ["calculate_fibonacci", "mandelbrot_checksum"],
    threads: 0
});

const rootNode = document.getElementById("root");
if (!rootNode) {
    throw new Error("missing required element: #root");
}

createRoot(rootNode).render(
    <StrictMode>
        <App worker={worker} />
    </StrictMode>
);

window.addEventListener("beforeunload", () => worker.terminate(), { once: true });