import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { nodeResolve } from "@rollup/plugin-node-resolve";
import typescript from "@rollup/plugin-typescript";
import serve from "rollup-plugin-serve";
import { orxParallelWasm } from "orx-parallel-wasm/rollup";

const isWatch = Boolean(process.env.ROLLUP_WATCH);

// Rollup has no built-in HTML/CSS pipeline, so copy the static files verbatim.
function copyStaticFiles() {
    return {
        name: "copy-static-files",
        async generateBundle() {
            this.emitFile({ type: "asset", fileName: "index.html", source: await readFile(resolve("index.html"), "utf8") });
            this.emitFile({ type: "asset", fileName: "style.css", source: await readFile(resolve("style.css"), "utf8") });
        }
    };
}

export default {
    input: "src/main.ts",
    output: {
        dir: "dist",
        entryFileNames: "assets/main.js",
        format: "es",
        sourcemap: isWatch
    },
    plugins: [
        nodeResolve(),
        typescript(),
        orxParallelWasm({
            bindings: "../wasm_bindings"
        }),
        copyStaticFiles(),
        // No live-reload plugin: it serves its client script from its own origin, which
        // COEP `require-corp` blocks. Rollup still rebuilds on change; refresh manually.
        isWatch && serve({
            contentBase: "dist",
            port: 8083,
            headers: {
                "Cross-Origin-Opener-Policy": "same-origin",
                "Cross-Origin-Embedder-Policy": "require-corp"
            }
        })
    ]
};
