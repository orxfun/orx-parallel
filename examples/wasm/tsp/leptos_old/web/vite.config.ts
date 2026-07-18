import { defineConfig } from "vite";

export default defineConfig({
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