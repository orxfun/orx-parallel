import init from "../pkg/orx_parallel_wasm_tsp_leptos.js";
import "./search-runner.ts";
import "../styles.css";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import "highlight.js/styles/github-dark.css";

hljs.registerLanguage("rust", rust);

void init().then(() => {
    hljs.highlightAll();
});