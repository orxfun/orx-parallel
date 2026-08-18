import { buildWasm, resolveThreads } from "./build.js";
import { fileURLToPath } from "node:url";

const packageRoot = fileURLToPath(new URL("..", import.meta.url));

export function orxParallelWasm(options) {
    let buildPromise;
    const threads = options.threads ?? resolveThreads();

    return {
        name: "orx-parallel-wasm",
        config(config) {
            return {
                server: {
                    fs: {
                        allow: [config.root ?? process.cwd(), packageRoot]
                    }
                },
                define: {
                    __ORX_PARALLEL_MAX_NUM_THREADS__: JSON.stringify(threads)
                }
            };
        },
        buildStart() {
            buildPromise ??= buildWasm(options);
            return buildPromise;
        },
        configureServer(server) {
            server.middlewares.use((_request, response, next) => {
                response.setHeader("Cross-Origin-Opener-Policy", "same-origin");
                response.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
                next();
            });
        }
    };
}