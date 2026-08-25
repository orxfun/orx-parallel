import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import "highlight.js/styles/github-dark.css";
import "../style.css";
import init from "../pkg/wasm_bindings.js";
import { App } from "./App";
import { createSearchWorker } from "./search-runner";

// 0 means "use all available threads"; passed to createSearchWorker and used to size the threads input
const num_threads = 0;
const MAX_THREADS = 32;

hljs.registerLanguage("rust", rust);

// num_threads === 0 means "all available threads", so fall back to hardwareConcurrency
const maxThreads = num_threads > 0 ? num_threads : (navigator.hardwareConcurrency || MAX_THREADS);

async function bootstrap() {
    await init();
    const searchWorker = await createSearchWorker(num_threads);

    const rootNode = document.getElementById("root");
    if (!rootNode) {
        searchWorker.terminate();
        throw new Error("missing required element: #root");
    }

    createRoot(rootNode).render(
        <StrictMode>
            <App searchWorker={searchWorker} maxThreads={maxThreads} />
        </StrictMode>
    );

    window.addEventListener("beforeunload", () => searchWorker.terminate(), { once: true });
}

void bootstrap();
