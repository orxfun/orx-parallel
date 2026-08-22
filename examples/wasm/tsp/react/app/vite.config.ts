import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { orxParallelWasm } from "orx-parallel-wasm/vite";

export default defineConfig({
    plugins: [
        react(),
        orxParallelWasm({
            bindings: ["../wasm_bindings"],
        })
    ],
    base: "./",
    worker: {
        format: "es"
    }
});
