import { createReadStream, promises as fs } from "node:fs";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..", "dist");
const port = Number(process.env.PORT || 4173);

const mimeTypes = new Map([
    [".html", "text/html; charset=utf-8"],
    [".js", "text/javascript; charset=utf-8"],
    [".css", "text/css; charset=utf-8"],
    [".json", "application/json; charset=utf-8"],
    [".wasm", "application/wasm"],
    [".svg", "image/svg+xml"],
    [".png", "image/png"],
    [".jpg", "image/jpeg"],
    [".jpeg", "image/jpeg"],
    [".gif", "image/gif"],
    [".ico", "image/x-icon"]
]);

function setIsolationHeaders(response) {
    response.setHeader("Cross-Origin-Opener-Policy", "same-origin");
    response.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
}

function sendNotFound(response) {
    response.statusCode = 404;
    response.end("Not found");
}

async function fileExists(filePath) {
    try {
        await fs.access(filePath);
        return true;
    } catch {
        return false;
    }
}

const server = http.createServer(async (request, response) => {
    setIsolationHeaders(response);

    if (!request.url) {
        sendNotFound(response);
        return;
    }

    const requestUrl = new URL(request.url, `http://${request.headers.host}`);
    const requestedPath = decodeURIComponent(requestUrl.pathname);
    const absolutePath = path.resolve(rootDir, `.${requestedPath}`);

    if (!absolutePath.startsWith(rootDir)) {
        sendNotFound(response);
        return;
    }

    let filePath = absolutePath;

    if ((await fileExists(filePath)) && (await fs.stat(filePath)).isDirectory()) {
        filePath = path.join(filePath, "index.html");
    }

    if (!(await fileExists(filePath))) {
        filePath = path.join(rootDir, "index.html");
    }

    if (!(await fileExists(filePath))) {
        sendNotFound(response);
        return;
    }

    const contentType = mimeTypes.get(path.extname(filePath).toLowerCase()) || "application/octet-stream";
    response.statusCode = 200;
    response.setHeader("Content-Type", contentType);
    createReadStream(filePath).pipe(response);
});

server.listen(port, () => {
    console.log(`Serving dist/ at http://localhost:${port}`);
});