import { defineConfig } from "vite";
import { orxParallelWasm } from "orx-parallel-web/vite";

export default defineConfig({
    base: "./",
    plugins: [
        orxParallelWasm({
            bindings: "../components",
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
    worker: {
        format: "es"
    }
});
