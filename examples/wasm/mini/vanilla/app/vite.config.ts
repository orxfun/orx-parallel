import { defineConfig } from "vite";
import { orxParallelWasm } from "orx-parallel-wasm/vite";

export default defineConfig({
    base: "./",
    plugins: [
        orxParallelWasm({
            bindings: "../wasm_bindings",
            outDir: "./pkg",
            threads: 0
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
