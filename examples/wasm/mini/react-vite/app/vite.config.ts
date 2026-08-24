import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { orxParallelWasm } from "orx-parallel-wasm/vite";

export default defineConfig({
    base: "./",
    plugins: [
        react(),
        orxParallelWasm({
            bindings: "../wasm_bindings"
        })
    ],
    server: {
        headers: {
            "Cross-Origin-Opener-Policy": "same-origin",
            "Cross-Origin-Embedder-Policy": "require-corp"
        }
    },
    worker: {
        format: "es"
    }
});