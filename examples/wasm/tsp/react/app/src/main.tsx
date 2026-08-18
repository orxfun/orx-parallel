import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import "highlight.js/styles/github-dark.css";
import "../style.css";
import init from "../pkg/wasm_bindings.js";
import { App } from "./App";
import { createSearchWorker } from "./search-runner";

hljs.registerLanguage("rust", rust);

async function bootstrap() {
    await init();
    const searchWorker = await createSearchWorker(0);

    const rootNode = document.getElementById("root");
    if (!rootNode) {
        searchWorker.terminate();
        throw new Error("missing required element: #root");
    }

    createRoot(rootNode).render(
        <StrictMode>
            <App searchWorker={searchWorker} />
        </StrictMode>
    );

    window.addEventListener("beforeunload", () => searchWorker.terminate(), { once: true });
}

void bootstrap();
