import { cp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { build } from "esbuild";

const appRoot = new URL(".", import.meta.url).pathname;
const distDir = resolve(appRoot, "dist");

await rm(distDir, { recursive: true, force: true });
await mkdir(resolve(distDir, "assets"), { recursive: true });

const indexHtml = await readFile(resolve(appRoot, "index.html"), "utf8");
await writeFile(resolve(distDir, "index.html"), indexHtml.replace("./src/main.ts", "./assets/main.js"));
await cp(resolve(appRoot, "style.css"), resolve(distDir, "style.css"));
await cp(resolve(appRoot, "pkg"), resolve(distDir, "pkg"), { recursive: true });
await cp(resolve(appRoot, "node_modules/orx-parallel-wasm/dist/worker.js"), resolve(distDir, "assets/worker.js"));

const packageMetadata = JSON.parse(await readFile(resolve(distDir, "pkg/orx-parallel-wasm.json"), "utf8"));
const workerFiles = packageMetadata.workerHelpers.flatMap((workerHelper) => [
    workerHelper,
    workerHelper.replace("worker_helpers.js", "wasm_web_start_workers.js")
]);
for (const workerFile of workerFiles) {
    const workerPath = resolve(distDir, "pkg", workerFile);
    const workerSource = await readFile(workerPath, "utf8");
    await writeFile(workerPath, workerSource.replace('import("../../../../..")', 'import("../../../../../wasm_bindings.js")'));
}

await build({
    entryPoints: [resolve(appRoot, "src/main.ts")],
    bundle: true,
    format: "esm",
    target: "es2020",
    outfile: resolve(distDir, "assets/main.js")
});