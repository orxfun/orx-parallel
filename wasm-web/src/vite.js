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
 * - `threads` (number, required): max number of threads for the parallel runtime (0 = auto).
 * - `bindings` (string, required): path to the Rust crate directory containing `Cargo.toml` to build.
 * - `outDir` (string, optional): directory where `wasm-pack` should write the package (defaults to `./pkg`).
 * - `bindingsFile` (string, optional): explicit package entry filename to use instead of auto-detection.
 *
 * Returns a Vite plugin object.
 */
export function orxParallelWasm(options) {
    if (options?.bindings === undefined) {
        throw new Error(
            "`bindings` is required: path to the Rust crate directory containing Cargo.toml that `wasm-pack` should build (for example '../wasm_bindings'). This plugin always builds the crate with `wasm-pack` during the build step; prebuilt packages are not accepted here. You may run the prepare script separately if you need to work with an existing pkg directory. `outDir` is optional — when omitted the plugin defaults to './pkg'."
        );
    }

    let buildPromise;
    let resolvedConfig;
    // normalized list of crate paths to build
    const bindingsList = Array.isArray(options.bindings) ? options.bindings : [options.bindings];
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

                    let perOut;
                    if (bindingsList.length === 1) {
                        perOut = options.outDir ?? './pkg';
                    } else {
                        // when building multiple crates, place outputs under outDir/<crateName>
                        const baseOut = options.outDir ?? './pkg';
                        perOut = pathResolve(process.cwd(), baseOut, crateName);
                    }

                    // call buildWasm with a copy of options for this crate
                    await buildWasm({
                        bindings: bindingPath,
                        outDir: perOut,
                        bindingsFile: options.bindingsFile,
                        threads: options.threads
                    });

                    pkgDirs.push(pathResolve(process.cwd(), perOut));
                }

                return undefined;
            })();

            return buildPromise;
        },
        async writeBundle() {
            // ensure wasm build completed
            try {
                await (buildPromise || Promise.resolve());
            } catch (e) {
                // ignore
            }

            try {
                const consumerOut = (resolvedConfig && resolvedConfig.build && resolvedConfig.build.outDir) || "dist";
                const distDir = pathResolve(process.cwd(), consumerOut);

                // copy each produced pkg output into dist/assets so all wasm/js/snippets are present
                const destAssets = pathResolve(distDir, "assets");
                await mkdir(destAssets, { recursive: true });
                for (const pd of (pkgDirs.length ? pkgDirs : [pathResolve(process.cwd(), options.outDir ?? './pkg')])) {
                    try {
                        await cp(pd, destAssets, { recursive: true, force: true });
                    } catch (e) {
                        // ignore if pkg doesn't exist
                    }
                }

                // write Cloudflare/Netlify-style _headers into dist so Pages will set COOP/COEP and wasm MIME
                try {
                    const headers = `/*
  Cross-Origin-Opener-Policy: same-origin
  Cross-Origin-Embedder-Policy: require-corp

/assets/*.wasm
  Content-Type: application/wasm
`;
                    await writeFile(pathResolve(distDir, "_headers"), headers, "utf8");
                } catch (e) {
                    // ignore
                }
                // Post-process all copied snippet JS files to ensure they import the package entry.
                // The entry must be the verbatim wasm-bindgen glue of *this* pkg: a hashed Rollup chunk may be
                // tree-shaken, and a leftover glue of another crate would instantiate the wasm with wrong imports.
                try {
                    const fs = await import('node:fs/promises');
                    const assetFiles = await fs.readdir(destAssets).catch(() => []);

                    // Build a deterministic map of crate -> entry JS filename.
                    // For each produced pkg dir prefer `package.json`'s `main`,
                    // otherwise pick a JS that contains the crate basename, or any JS.
                    const crateEntryMap = Object.create(null);
                    for (let i = 0; i < (pkgDirs.length || 0); i++) {
                        const pd = pkgDirs[i];
                        const crate = crateNames[i] ?? pathBasename(String(pd)).replace(/[^A-Za-z0-9_\-]/g, '_');
                        const entriesInPkg = await fs.readdir(pd).catch(() => []);

                        let declaredMain;
                        try {
                            const pj = await readFile(pathResolve(pd, 'package.json'), 'utf8')
                                .then(text => JSON.parse(text))
                                .catch(() => undefined);
                            if (pj && typeof pj.main === 'string') declaredMain = pj.main;
                        } catch (e) {
                            declaredMain = undefined;
                        }

                        let candidate;
                        if (declaredMain && entriesInPkg.includes(declaredMain)) candidate = declaredMain;
                        if (!candidate) candidate = entriesInPkg.find(n => n.includes(crate) && n.endsWith('.js'));
                        if (!candidate) candidate = entriesInPkg.find(n => n.endsWith('.js'));

                        // Prefer the filename as present in the final assets directory when possible.
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

                    const replacement = `import(new URL('../../../../..', import.meta.url).href + '${pkgMain}')`;

                    async function visit(dir) {
                        const entries = await fs.readdir(dir, { withFileTypes: true });
                        for (const ent of entries) {
                            const p = pathResolve(dir, ent.name);
                            if (ent.isDirectory()) {
                                await visit(p);
                            } else if (ent.isFile() && p.endsWith('.js')) {
                                try {
                                    let content = await readFile(p, 'utf8');

                                    // compute a relative URL from this file to the assets directory
                                    // e.g. '../../../../../' so the snippet can import the asset file by name
                                    const rel = pathResolve(p).replace(/\\/g, '/');
                                    const assetsPath = destAssets.replace(/\\/g, '/');
                                    // dirname of the file
                                    const fileDir = pathResolve(p, '..').replace(/\\/g, '/');
                                    // compute relative path from file's dir to assets dir (posix-style)
                                    let relToAssets = pathRelative(fileDir, assetsPath).replace(/\\/g, '/');
                                    if (!relToAssets || relToAssets === '') relToAssets = '.';
                                    if (!relToAssets.endsWith('/')) relToAssets += '/';

                                    const fileReplacement = `import(new URL('${relToAssets}', import.meta.url).href + '${pkgMain}')`;

                                    content = content.split('import("../../../../..")').join(fileReplacement);
                                    content = content.split("import('../../../../..')").join(fileReplacement);
                                    content = content.replace(/import\(\s*['\"]?\.\.\/\.\.\/\.\.\.\s*['\"]?\s*\)/g, fileReplacement);
                                    await writeFile(p, content, 'utf8');
                                } catch (e) {
                                    // ignore per-file errors
                                }
                            }
                        }
                    }

                    const snippetsRoot = pathResolve(distDir, 'assets', 'snippets');
                    await visit(snippetsRoot).catch(() => { });

                    // Ensure there's a stable assets/index.js shim that forwards to the pkgMain
                    try {
                        const shimPath = pathResolve(destAssets, 'index.js');
                        const shimContent = `import './${pkgMain}';\nexport * from './${pkgMain}';\n`;
                        await writeFile(shimPath, shimContent, 'utf8');
                    } catch (e) {
                        // ignore
                    }
                    // Duplicate common worker files to stable filenames (worker, worker_helpers)
                    try {
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
                            try {
                                const srcPath = pathResolve(destAssets, src);
                                const destPath = pathResolve(destAssets, dest);
                                await cp(srcPath, destPath, { force: true });
                            } catch (e) {
                                // ignore per-file duplicate errors
                            }
                        }
                    } catch (e) {
                        // ignore
                    }
                    // Create explicit shims for each built crate named by its basename
                    try {
                        for (let i = 0; i < (crateNames.length || 0); i++) {
                            const crate = crateNames[i];
                            const candidate = crateEntryMap[crate] || assetFiles.find(n => n.endsWith('.js')) || pkgMain;
                            if (candidate) {
                                const shimName = `${crate}.js`;
                                const shimPath = pathResolve(destAssets, shimName);
                                const shimContent = `import './${candidate}';\nexport * from './${candidate}';\n`;
                                try {
                                    await writeFile(shimPath, shimContent, 'utf8');
                                } catch (e) {
                                    // ignore per-file errors
                                }
                            }
                        }
                        // ensure a compatibility shim `wasm_bindings.js` pointing at first crate's entry
                        if (crateNames.length > 0) {
                            const first = crateNames[0];
                            const candidate = crateEntryMap[first] || pkgMain || 'index.js';
                            const shimPath = pathResolve(destAssets, 'wasm_bindings.js');
                            const shimContent = `import './${candidate}';\nexport * from './${candidate}';\n`;
                            await writeFile(shimPath, shimContent, 'utf8').catch(() => { /* ignore */ });
                        }
                    } catch (e) {
                        // ignore
                    }
                    // Create wrapper modules for wasm-imported JS namespaces like `components_bg.js`
                    try {
                        const wasmFiles = assetFiles.filter(n => /_?bg[-_]?.*\.wasm$/.test(n));
                        for (const wasmName of wasmFiles) {
                            // derive base like 'components_bg' from 'components_bg-XXX.wasm' or 'components_bg.wasm'
                            const base = wasmName.replace(/(-[A-Za-z0-9_]+)?\.wasm$/, '').replace(/-bg$/, '_bg');
                            // find a JS candidate that likely provides the glue by name match
                            const jsCandidate = assetFiles.find(n => n.includes(base.replace('_bg', '')) && n.endsWith('.js'))
                                || assetFiles.find(n => n.startsWith(base.replace('_bg', '')) && n.endsWith('.js'))
                                || assetFiles.find(n => n.endsWith('.js'));
                            if (jsCandidate) {
                                const wrapperName = `${base}.js`;
                                const wrapperPath = pathResolve(destAssets, wrapperName);
                                const wrapperContent = `export * from './${jsCandidate}';\nexport { default } from './${jsCandidate}';\n`;
                                try {
                                    await writeFile(wrapperPath, wrapperContent, 'utf8');
                                } catch (e) {
                                    // ignore per-file errors
                                }
                            }
                        }
                    } catch (e) {
                        // ignore
                    }
                } catch (e) {
                    // ignore
                }

                // Ensure there's a stable assets/index.js shim that forwards to the real built entry
                try {
                    const fs = await import('node:fs/promises');
                    const assetFiles = await fs.readdir(destAssets);
                    // prefer a hashed Vite entry like index-*.js, or wasm pkg entry wasm_bindings.js
                    const candidate = assetFiles.find(n => /^index-.*\.js$/.test(n)) || assetFiles.find(n => n === 'wasm_bindings.js');
                    if (candidate) {
                        const shimPath = pathResolve(destAssets, 'index.js');
                        const shimContent = `import './${candidate}';\nexport * from './${candidate}';\n`;
                        await writeFile(shimPath, shimContent, 'utf8');
                    }
                } catch (e) {
                    // ignore
                }
            } catch (e) {
                // copying is best-effort; consumer build may not produce pkg
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