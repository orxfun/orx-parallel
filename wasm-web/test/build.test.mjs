import assert from "node:assert/strict";
import { mkdtemp, readFile, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { prepareWasm, resolveThreads } from "../src/build.js";

test("prepareWasm copies worker helpers and writes a manifest", async () => {
    const root = await mkdtemp(join(tmpdir(), "orx-parallel-web-"));
    const snippet = join(root, "snippets", "orx-parallel", "src", "pool", "pool_impl");
    await mkdir(snippet, { recursive: true });
    await writeFile(join(snippet, "wasm_web_start_workers.js"), "export {};");

    const manifest = await prepareWasm({ outDir: root, threads: 4 });

    assert.equal(manifest.threads, 4);
    assert.equal(await readFile(join(snippet, "worker_helpers.js"), "utf8"), "export {};");
    assert.deepEqual(JSON.parse(await readFile(join(root, "orx-parallel-web.json"), "utf8")), manifest);
});

test("resolveThreads validates the build environment value", () => {
    assert.equal(resolveThreads("4"), 4);
    assert.equal(resolveThreads("0"), 0);
    assert.throws(() => resolveThreads("-1"), /non-negative integer/);
    assert.throws(() => resolveThreads("many"), /non-negative integer/);
});