import { defineConfig } from "vite";
import { orxParallelWasm } from "orx-parallel-wasm/vite";

export default defineConfig({
    base: "./",
    plugins: [
        orxParallelWasm({
            bindings: ["../wasm_bindings"],
        })
    ],
    worker: {
        format: "es"
    }
});
