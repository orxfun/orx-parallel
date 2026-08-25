import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, normalize, resolve } from "node:path";

const root = resolve(new URL(".", import.meta.url).pathname, "dist");
const port = Number(process.env.PORT ?? 8080);
const contentTypes = {
    ".css": "text/css",
    ".html": "text/html",
    ".js": "text/javascript",
    ".json": "application/json",
    ".wasm": "application/wasm"
};

createServer(async (request, response) => {
    const requestedPath = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
    const relativePath = requestedPath === "/" ? "/index.html" : normalize(requestedPath);
    const filePath = resolve(root, `.${relativePath}`);

    if (filePath !== root && !filePath.startsWith(`${root}/`)) {
        response.writeHead(403);
        response.end("Forbidden");
        return;
    }

    try {
        const body = await readFile(filePath);
        response.writeHead(200, {
            "Content-Type": contentTypes[extname(relativePath)] ?? "application/octet-stream",
            "Cross-Origin-Opener-Policy": "same-origin",
            "Cross-Origin-Embedder-Policy": "require-corp"
        });
        response.end(body);
    } catch {
        response.writeHead(404);
        response.end("Not found");
    }
}).listen(port, () => {
    console.log(`Manual WASM example: http://localhost:${port}`);
});