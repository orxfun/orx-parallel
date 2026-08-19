import assert from "node:assert/strict";
import { mkdtemp, readFile, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { prepareWasm, normalizeThreads } from "../src/build.js";

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

test("normalizeThreads validates the configured value", () => {
    assert.equal(normalizeThreads("4"), 4);
    assert.equal(normalizeThreads(0), 0);
    assert.throws(() => normalizeThreads(-1), /non-negative integer/);
    assert.throws(() => normalizeThreads("many"), /non-negative integer/);
});