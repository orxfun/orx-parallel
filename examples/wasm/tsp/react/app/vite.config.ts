import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { orxParallelWasm } from "orx-parallel-web/vite";

export default defineConfig({
    plugins: [
        react(),
        orxParallelWasm({
            bindings: "../wasm_bindings",
            outDir: "./pkg",
            schemas: {
                run_search: {
                    args: [
                        { type: "number" },
                        { type: "bigint" },
                        { type: "number" },
                        { type: "number" },
                        {
                            type: "array",
                            items: {
                                type: "object",
                                properties: {
                                    x: { type: "number" },
                                    y: { type: "number" }
                                }
                            }
                        }
                    ]
                }
            }
        })
    ],
    base: "./",
    worker: {
        format: "es"
    }
});
