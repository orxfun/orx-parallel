import { buildWasm } from "./build.js";
import { fileURLToPath } from "node:url";
import { cp, mkdir, writeFile } from "node:fs/promises";
import { readFile } from "node:fs/promises";
import { resolve as pathResolve, basename as pathBasename, relative as pathRelative } from "node:path";

const packageRoot = fileURLToPath(new URL("..", import.meta.url));

/**
 * Vite plugin that builds a Rust `wasm-pack` crate, prepares the generated
 * package, and integrates the resulting wasm/JS assets into the consumer
 * build (copies into `dist/assets`, writes COOP/COEP headers, and creates
 * stable shims for worker/wasm artifacts).
 *
 * Options (object):
 * - `bindings` (string | string[], required): path, or array of paths, to Rust crate directory(ies)
 *   containing `Cargo.toml` that `wasm-pack` should build (e.g. `"../wasm_bindings"` or
 *   `["../wasm_bindings","../components"]`).
 * - `bindingsFile` (string, optional): explicit package entry filename to use instead of auto-detection.
 *
 * Returns a Vite plugin object.
 */
export function orxParallelWasm(options) {
    if (options?.bindings === undefined) {
        throw new Error(
            "`bindings` is required: a path or array of paths to Rust crate directory(ies) (each with Cargo.toml) that `wasm-pack` will build, e.g. '../wasm_bindings' or ['../wasm_bindings','../components']. This plugin builds the listed crates during the build step (prebuilt pkg directories are not accepted). Output directories default to './pkg' (per-crate subdirs when multiple bindings are used). Use `bindingsFile` to override the package entry filename."
        );
    }

    let buildPromise;
    let resolvedConfig;
    // normalized list of crate paths to build (must be non-empty)
    const bindingsList = Array.isArray(options.bindings) ? options.bindings : [options.bindings];
    if (!Array.isArray(bindingsList) || bindingsList.length === 0) {
        throw new Error("`bindings` must be a non-empty string or array of crate paths, e.g. ['../wasm_bindings'] or '../wasm_bindings'.");
    }
    const pkgDirs = [];
    const crateNames = [];

    return {
        name: "orx-parallel-wasm",
        config(config) {
            return {
                server: {
                    fs: {
                        allow: [config.root ?? process.cwd(), packageRoot]
                    }
                },
            };
        },
        configResolved(config) {
            resolvedConfig = config;
        },
        buildStart() {
            buildPromise ??= (async () => {
                // Build each crate in `bindingsList` sequentially. For a single binding
                // respect options.outDir; for multiple bindings create per-crate subdirs.
                for (let i = 0; i < bindingsList.length; i++) {
                    const bindingPath = bindingsList[i];
                    const crateName = pathBasename(String(bindingPath)).replace(/[^A-Za-z0-9_\-]/g, '_');
                    crateNames.push(crateName);

                    // compute per-crate output: single binding -> ./pkg, multiple -> ./pkg/<crateName>
                    let perOut;
                    if (bindingsList.length === 1) {
                        perOut = './pkg';
                    } else {
                        const baseOut = './pkg';
                        perOut = pathResolve(process.cwd(), baseOut, crateName);
                    }

                    // call buildWasm with a copy of options for this crate
                    await buildWasm({
                        bindings: bindingPath,
                        outDir: perOut,
                        bindingsFile: options.bindingsFile,
                    });

                    pkgDirs.push(pathResolve(process.cwd(), perOut));
                }

                return undefined;
            })();

            return buildPromise;
        },
        async writeBundle() {
            // ensure wasm build completed (propagate errors)
            await (buildPromise || Promise.resolve());

            const consumerOut = (resolvedConfig && resolvedConfig.build && resolvedConfig.build.outDir) || "dist";
            const distDir = pathResolve(process.cwd(), consumerOut);

            // copy each produced pkg output into dist/assets so all wasm/js/snippets are present
            const destAssets = pathResolve(distDir, "assets");
            await mkdir(destAssets, { recursive: true });

            if (!pkgDirs.length) {
                throw new Error('No package outputs were produced for the configured `bindings`. Ensure the Rust crates were built.');
            }

            for (const pd of pkgDirs) {
                // Let CP throw if the pkg dir is missing or copy fails
                await cp(pd, destAssets, { recursive: true, force: true });
            }

            // write Cloudflare/Netlify-style _headers into dist so Pages will set COOP/COEP and wasm MIME
            const headers = `/*
  Cross-Origin-Opener-Policy: same-origin
  Cross-Origin-Embedder-Policy: require-corp

/assets/*.wasm
  Content-Type: application/wasm
`;
            await writeFile(pathResolve(distDir, "_headers"), headers, "utf8");

            // Post-process all copied snippet JS files to ensure they import the package entry.
            const fs = await import('node:fs/promises');
            const assetFiles = await fs.readdir(destAssets);
            if (!assetFiles.length) throw new Error(`No files found in assets after copying packages from: ${pkgDirs.join(',')}`);

            // Build a deterministic map of crate -> entry JS filename.
            const crateEntryMap = Object.create(null);
            for (let i = 0; i < pkgDirs.length; i++) {
                const pd = pkgDirs[i];
                const crate = crateNames[i] ?? pathBasename(String(pd)).replace(/[^A-Za-z0-9_\-]/g, '_');
                const entriesInPkg = await fs.readdir(pd);

                const pj = await readFile(pathResolve(pd, 'package.json'), 'utf8')
                    .then(text => JSON.parse(text))
                    .catch(() => undefined);
                const declaredMain = (pj && typeof pj.main === 'string') ? pj.main : undefined;

                let candidate;
                if (declaredMain && entriesInPkg.includes(declaredMain)) candidate = declaredMain;
                if (!candidate) candidate = entriesInPkg.find(n => n.includes(crate) && n.endsWith('.js'));
                if (!candidate) candidate = entriesInPkg.find(n => n.endsWith('.js'));

                if (candidate && assetFiles.includes(candidate)) {
                    crateEntryMap[crate] = candidate;
                } else if (candidate) {
                    crateEntryMap[crate] = candidate;
                } else {
                    crateEntryMap[crate] = undefined;
                }
            }

            // Primary pkgMain is the entry for the first crate or a fallback JS in assets
            const primaryCrate = crateNames[0];
            const pkgMain = (primaryCrate && crateEntryMap[primaryCrate])
                || assetFiles.find(n => n.endsWith('.js'))
                || 'index.js';

            async function visit(dir) {
                const entries = await fs.readdir(dir, { withFileTypes: true });
                for (const ent of entries) {
                    const p = pathResolve(dir, ent.name);
                    if (ent.isDirectory()) {
                        await visit(p);
                    } else if (ent.isFile() && p.endsWith('.js')) {
                        let content = await readFile(p, 'utf8');

                        // compute relative path from file's dir to assets dir (posix-style)
                        const fileDir = pathResolve(p, '..').replace(/\\/g, '/');
                        const assetsPath = destAssets.replace(/\\/g, '/');
                        let relToAssets = pathRelative(fileDir, assetsPath).replace(/\\/g, '/');
                        if (!relToAssets || relToAssets === '') relToAssets = '.';
                        if (!relToAssets.endsWith('/')) relToAssets += '/';

                        const fileReplacement = `import(new URL('${relToAssets}', import.meta.url).href + '${pkgMain}')`;

                        content = content.split('import("../../../../..")').join(fileReplacement);
                        content = content.split("import('../../../../..')").join(fileReplacement);
                        content = content.replace(/import\(\s*['\"]?\.\.\/\.\.\/\.\.\.\s*['\"]?\s*\)/g, fileReplacement);
                        await writeFile(p, content, 'utf8');
                    }
                }
            }

            const snippetsRoot = pathResolve(distDir, 'assets', 'snippets');
            // ensure snippets exist — treat absence as error
            const stat = await fs.stat(snippetsRoot).catch(() => null);
            if (!stat || !stat.isDirectory()) {
                throw new Error(`Expected snippets directory at ${snippetsRoot} but none found`);
            }
            await visit(snippetsRoot);

            // Ensure there's a stable assets/index.js shim that forwards to the pkgMain
            const shimPath = pathResolve(destAssets, 'index.js');
            const shimContent = `import './${pkgMain}';\nexport * from './${pkgMain}';\n`;
            await writeFile(shimPath, shimContent, 'utf8');

            // Duplicate common worker files to stable filenames (worker, worker_helpers)
            const duplicates = [];
            for (const f of assetFiles) {
                if (/^worker[-_].*\.js$/.test(f) || /^workerHelpers[-_].*\.js$/i.test(f)) {
                    duplicates.push({ src: f, dest: 'worker.js' });
                }
                if (/^worker_helpers[-_].*\.js$/.test(f)) {
                    duplicates.push({ src: f, dest: 'worker_helpers.js' });
                }
            }
            for (const { src, dest } of duplicates.filter(d => d.dest !== pkgMain)) {
                const srcPath = pathResolve(destAssets, src);
                const destPath = pathResolve(destAssets, dest);
                await cp(srcPath, destPath, { force: true });
            }

            // Create explicit shims for each built crate named by its basename
            for (let i = 0; i < crateNames.length; i++) {
                const crate = crateNames[i];
                const candidate = crateEntryMap[crate] || assetFiles.find(n => n.endsWith('.js')) || pkgMain;
                if (candidate) {
                    const shimName = `${crate}.js`;
                    const shimPath = pathResolve(destAssets, shimName);
                    const shimContent = `import './${candidate}';\nexport * from './${candidate}';\n`;
                    await writeFile(shimPath, shimContent, 'utf8');
                }
            }
            // ensure a compatibility shim `wasm_bindings.js` pointing at first crate's entry
            if (crateNames.length > 0) {
                const first = crateNames[0];
                const candidate = crateEntryMap[first] || pkgMain || 'index.js';
                const wasmShimPath = pathResolve(destAssets, 'wasm_bindings.js');
                const wasmShimContent = `import './${candidate}';\nexport * from './${candidate}';\n`;
                await writeFile(wasmShimPath, wasmShimContent, 'utf8');
            }

            // Create wrapper modules for wasm-imported JS namespaces like `*_bg.wasm`
            const wasmFiles = assetFiles.filter(n => /_?bg[-_]?.*\.wasm$/.test(n));
            for (const wasmName of wasmFiles) {
                const base = wasmName.replace(/(-[A-Za-z0-9_]+)?\.wasm$/, '').replace(/-bg$/, '_bg');
                const jsCandidate = assetFiles.find(n => n.includes(base.replace('_bg', '')) && n.endsWith('.js'))
                    || assetFiles.find(n => n.startsWith(base.replace('_bg', '')) && n.endsWith('.js'))
                    || assetFiles.find(n => n.endsWith('.js'));
                if (jsCandidate) {
                    const wrapperName = `${base}.js`;
                    const wrapperPath = pathResolve(destAssets, wrapperName);
                    const wrapperContent = `export * from './${jsCandidate}';\nexport { default } from './${jsCandidate}';\n`;
                    await writeFile(wrapperPath, wrapperContent, 'utf8');
                }
            }
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