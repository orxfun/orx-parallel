import { defineConfig } from "vite";

export default defineConfig({
    base: "./",
    envPrefix: ["VITE_", "ORX_PARALLEL_"],
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
