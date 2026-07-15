import { defineConfig } from "vite";

export default defineConfig({
    envPrefix: ["VITE_", "PAR_POOL_", "PAR_NUM_"],
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
