import assert from "node:assert/strict";
import test from "node:test";
import { orxParallelWasm } from "../src/vite.js";

test("Vite plugin allows the app and local package roots", () => {
    const plugin = orxParallelWasm({});
    const config = plugin.config({ root: "/example/app" });

    assert.equal(config.server.fs.allow[0], "/example/app");
    assert.match(config.server.fs.allow[1], /wasm-web\/?$/);
});

test("Vite middleware adds cross-origin isolation headers", () => {
    const plugin = orxParallelWasm({});
    const headers = new Map();
    let middleware;
    plugin.configureServer({
        middlewares: {
            use(value) {
                middleware = value;
            }
        }
    });

    middleware({}, { setHeader: (name, value) => headers.set(name, value) }, () => undefined);

    assert.equal(headers.get("Cross-Origin-Opener-Policy"), "same-origin");
    assert.equal(headers.get("Cross-Origin-Embedder-Policy"), "require-corp");
});