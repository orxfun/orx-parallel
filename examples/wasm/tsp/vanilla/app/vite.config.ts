import { defineConfig } from "vite";
import { orxParallelWasm } from "orx-parallel-wasm/vite";

export default defineConfig({
    base: "./",
    plugins: [
        orxParallelWasm({
            bindings: ["../wasm_bindings"],
        })
    ],
    optimizeDeps: {
        exclude: ["orx-parallel-wasm"]
    },
    worker: {
        format: "es"
    }
});
