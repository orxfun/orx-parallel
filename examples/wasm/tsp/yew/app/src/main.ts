import init, { start_app } from "../pkg/components.js";
import { createSearchWorker, terminateSearchWorker } from "./search-runner";
import "../style.css";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import "highlight.js/styles/github-dark.css";

hljs.registerLanguage("rust", rust);

// 0 means "use all available threads"; passed to createSearchWorker and used to size the threads input
const num_threads = 0;
const MAX_THREADS = 32;

(globalThis as typeof globalThis & { highlightCodeBlocks: () => void }).highlightCodeBlocks = () => {
    hljs.highlightAll();
};

void init().then(async () => {
    await createSearchWorker(num_threads);
    start_app(num_threads);
    window.addEventListener("beforeunload", terminateSearchWorker, { once: true });
});