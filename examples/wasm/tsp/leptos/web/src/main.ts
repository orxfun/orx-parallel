import init, { start_app } from "../pkg/ui.js";
import "./search-runner.ts";
import "../styles.css";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import "highlight.js/styles/github-dark.css";

hljs.registerLanguage("rust", rust);

void init().then(() => {
    start_app();
    hljs.highlightAll();
});
