import { buildWasm } from "./build.js";

export function orxParallelWasm(options) {
    let buildPromise;

    return {
        name: "orx-parallel-wasm",
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