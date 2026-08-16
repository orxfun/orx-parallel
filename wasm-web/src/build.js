import { execFileSync } from "node:child_process";
import { cp, mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, join, relative, resolve } from "node:path";

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

export function resolveThreads(value = process.env.ORX_PARALLEL_MAX_NUM_THREADS ?? "0") {
    const threads = Number(value);
    if (!Number.isInteger(threads) || threads < 0) {
        throw new Error("ORX_PARALLEL_MAX_NUM_THREADS must be a non-negative integer");
    }
    return threads;
}

export async function prepareWasm({ outDir, bindingsFile = "wasm_bindings.js", threads = resolveThreads() }) {
    const outputDir = resolve(outDir);
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
        bindingsUrl: `./${bindingsFile}`,
        threads,
        workerHelpers: workerSources.map((source) => relative(outputDir, join(dirname(source), "worker_helpers.js")))
    };
    await writeFile(join(outputDir, "orx-parallel-web.json"), `${JSON.stringify(manifest, null, 2)}\n`);
    return manifest;
}

export async function buildWasm({
    bindings,
    outDir,
    bindingsFile = "wasm_bindings.js",
    threads = resolveThreads(),
    wasmPack = "wasm-pack",
    rustupToolchain = "nightly",
    rustflags = DEFAULT_RUSTFLAGS
}) {
    if (bindings === undefined || outDir === undefined) {
        throw new Error("build requires bindings and outDir");
    }

    const outputDir = resolve(outDir);
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
        env: { ...process.env, RUSTUP_TOOLCHAIN: rustupToolchain, RUSTFLAGS: rustflags }
    });

    return prepareWasm({ outDir: outputDir, bindingsFile, threads });
}

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

async function main() {
    const mode = process.argv[2];
    const bindings = process.env.ORX_PARALLEL_WASM_BINDINGS ?? "../wasm_bindings";
    const outDir = process.env.ORX_PARALLEL_WASM_OUT_DIR ?? "../app/pkg";
    const options = {
        bindings,
        outDir,
        bindingsFile: process.env.ORX_PARALLEL_WASM_BINDINGS_FILE,
        threads: resolveThreads()
    };

    if (mode === "build") {
        await buildWasm(options);
    } else if (mode === "prepare") {
        await prepareWasm({ outDir, bindingsFile: options.bindingsFile, threads: options.threads });
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