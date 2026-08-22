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

function threadsCap(): number {
    return num_threads > 0 ? num_threads : (navigator.hardwareConcurrency || MAX_THREADS);
}

void init().then(async () => {
    await createSearchWorker(num_threads);
    start_app(threadsCap());
    hljs.highlightAll();
    window.addEventListener("beforeunload", terminateSearchWorker, { once: true });
});
