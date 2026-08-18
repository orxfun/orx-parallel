import init, { start_app } from "../pkg/components.js";
import { createSearchWorker, terminateSearchWorker } from "./search-runner";
import "../style.css";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import "highlight.js/styles/github-dark.css";

hljs.registerLanguage("rust", rust);

(globalThis as typeof globalThis & { highlightCodeBlocks: () => void }).highlightCodeBlocks = () => {
    hljs.highlightAll();
};

void init().then(async () => {
    await createSearchWorker(0);
    start_app();
    window.addEventListener("beforeunload", terminateSearchWorker, { once: true });
});