import init, { start_app } from "../pkg/components.js";
import "./search-runner.ts";
import "../style.css";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import "highlight.js/styles/github-dark.css";

hljs.registerLanguage("rust", rust);

(globalThis as typeof globalThis & { highlightCodeBlocks: () => void }).highlightCodeBlocks = () => {
    hljs.highlightAll();
};

void init().then(() => {
    start_app();
});