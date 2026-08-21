import { execFileSync } from "node:child_process";
import { cp, mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { dirname, join, relative, resolve, basename as pathBasename } from "node:path";

const DEFAULT_RUSTFLAGS = [
    "-C target-feature=+atomics",
    "-C link-arg=--shared-memory",
    "-C link-arg=--max-memory=1073741824",
    "-C link-arg=--import-memory",
    "-C link-arg=--export=__heap_base",
    "-C link-arg=--export=__wasm_init_tls",
    "-C link-arg=--export=__tls_size",
    "-C link-arg=--export=__tls_align",
    "-C link-arg=--export=__tls_base"
].join(" ");

/**
 * Prepare a wasm package directory for consumption by the plugin.
 * - Reads `package.json` (or uses `bindingsFile`) to determine the JS entry
 * - Copies worker helper sources and writes `orx-parallel-web.json` manifest
 *
 * @param {{outDir: string, bindingsFile?: string}} options
 * @returns {{bindingsUrl: string, workerHelpers: string[]}} manifest
 */
export async function prepareWasm({ outDir, bindingsFile }) {
    const outputDir = resolve(outDir);
    const entry = bindingsFile ?? await readFile(join(outputDir, "package.json"), "utf8")
        .then(text => JSON.parse(text).main)
        .catch(() => undefined)
        ?? "wasm_bindings.js";
    const snippetRoot = join(outputDir, "snippets");
    const workerSources = await findFiles(snippetRoot, "wasm_web_start_workers.js");

    if (workerSources.length === 0) {
        throw new Error(`no wasm_web_start_workers.js found under ${snippetRoot}`);
    }

    for (const source of workerSources) {
        const destination = join(dirname(source), "worker_helpers.js");
        await cp(source, destination);
    }

    const manifest = {
        bindingsUrl: `./${entry}`,
        workerHelpers: workerSources.map((source) => relative(outputDir, join(dirname(source), "worker_helpers.js")))
    };
    await writeFile(join(outputDir, "orx-parallel-web.json"), `${JSON.stringify(manifest, null, 2)}\n`);
    return manifest;
}

/**
 * Build a Rust crate with `wasm-pack` (web target) into `outDir` and then
 * prepare the produced package with `prepareWasm()`.
 *
 * @param {{bindings: string, outDir: string, bindingsFile?: string, wasmPack?: string, rustupToolchain?: string, rustflags?: string}} options
 * @returns {Promise<ReturnType<typeof prepareWasm>>}
 */
export async function buildWasm({
    bindings,
    outDir,
    bindingsFile,
    wasmPack = "wasm-pack",
    rustupToolchain = "nightly",
    rustflags = DEFAULT_RUSTFLAGS
}) {
    if (bindings === undefined || outDir === undefined) {
        throw new Error("build requires bindings and outDir");
    }

    const outputDir = resolve(outDir);
    // stale artifacts of a previously built crate would otherwise be copied into dist and picked up by workers
    await rm(outputDir, { recursive: true, force: true });
    await mkdir(outputDir, { recursive: true });
    execFileSync(wasmPack, [
        "build",
        resolve(bindings),
        "--target",
        "web",
        "--out-dir",
        outputDir,
        "--",
        "-Z",
        "build-std=panic_abort,std"
    ], {
        stdio: "inherit",
        env: {
            ...process.env,
            RUSTUP_TOOLCHAIN: rustupToolchain,
            CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS: rustflags
        }
    });

    return prepareWasm({ outDir: outputDir, bindingsFile });
}

/**
 * Recursively find files with the given `filename` under `directory`.
 * Returns an array of absolute paths to matching files.
 *
 * @param {string} directory
 * @param {string} filename
 * @returns {Promise<string[]>}
 */
async function findFiles(directory, filename) {
    const matches = [];
    let entries;
    try {
        entries = await readdir(directory, { withFileTypes: true });
    } catch (error) {
        if (error.code === "ENOENT") return matches;
        throw error;
    }

    for (const entry of entries) {
        const path = join(directory, entry.name);
        if (entry.isDirectory()) {
            matches.push(...await findFiles(path, filename));
        } else if (entry.isFile() && entry.name === filename) {
            matches.push(path);
        }
    }
    return matches;
}

/**
 * CLI entry for this script. Usage: `node src/build.js <build|prepare>`.
 * Reads env vars `ORX_PARALLEL_WASM_BINDINGS`, `ORX_PARALLEL_WASM_OUT_DIR`,
 * and `ORX_PARALLEL_WASM_BINDINGS_FILE` to configure the operation.
 */
async function main() {
    const mode = process.argv[2];
    const bindingsEnv = process.env.ORX_PARALLEL_WASM_BINDINGS;
    if (!bindingsEnv) {
        throw new Error('ORX_PARALLEL_WASM_BINDINGS must be set to the path (or list) of Rust crate(s) to build');
    }

    // Accept either a JSON array string or a comma-separated list
    let bindingsList;
    try {
        if (bindingsEnv.trim().startsWith('[')) {
            bindingsList = JSON.parse(bindingsEnv);
        } else {
            bindingsList = bindingsEnv.split(',').map(s => s.trim()).filter(Boolean);
        }
    } catch (e) {
        throw new Error('ORX_PARALLEL_WASM_BINDINGS must be a path, comma-separated list, or JSON array');
    }

    if (!Array.isArray(bindingsList) || bindingsList.length === 0) {
        throw new Error('ORX_PARALLEL_WASM_BINDINGS resolved to no bindings');
    }

    const baseOutDir = process.env.ORX_PARALLEL_WASM_OUT_DIR ?? "../app/pkg";
    const bindingsFile = process.env.ORX_PARALLEL_WASM_BINDINGS_FILE;

    if (mode === "build") {
        for (let i = 0; i < bindingsList.length; i++) {
            const binding = bindingsList[i];
            const perOut = (bindingsList.length === 1)
                ? baseOutDir
                : resolve(baseOutDir, pathBasename(String(binding)).replace(/[^A-Za-z0-9_\-]/g, '_'));
            await buildWasm({ bindings: binding, outDir: perOut, bindingsFile });
        }
    } else if (mode === "prepare") {
        for (let i = 0; i < bindingsList.length; i++) {
            const binding = bindingsList[i];
            const perOut = (bindingsList.length === 1)
                ? baseOutDir
                : resolve(baseOutDir, pathBasename(String(binding)).replace(/[^A-Za-z0-9_\-]/g, '_'));
            await prepareWasm({ outDir: perOut, bindingsFile });
        }
    } else {
        throw new Error("usage: node src/build.js <build|prepare>");
    }
}

if (import.meta.url === `file://${process.argv[1]}`) {
    main().catch((error) => {
        console.error(error.message);
        process.exitCode = 1;
    });
}