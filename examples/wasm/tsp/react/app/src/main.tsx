import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import hljs from "highlight.js/lib/core";
import rust from "highlight.js/lib/languages/rust";
import "highlight.js/styles/github-dark.css";
import "../style.css";
import init from "../pkg/wasm_bindings.js";
import { App } from "./App";

hljs.registerLanguage("rust", rust);

async function bootstrap() {
    await init();

    const rootNode = document.getElementById("root");
    if (!rootNode) {
        throw new Error("missing required element: #root");
    }

    createRoot(rootNode).render(
        <StrictMode>
            <App />
        </StrictMode>
    );
}

void bootstrap();
