import { defineConfig } from "vite";
import { orxParallelWasm } from "orx-parallel-wasm/vite";

export default defineConfig({
    base: "./",
    plugins: [
        orxParallelWasm({
            bindings: ["../wasm_bindings", "../components"]
        })
    ],
    worker: {
        format: "es"
    }
});
