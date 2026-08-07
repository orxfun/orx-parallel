import { defineConfig } from "vite";

export default defineConfig({
    base: "./",
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