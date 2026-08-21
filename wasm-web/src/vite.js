import { buildWasm } from "./build.js";
import { fileURLToPath } from "node:url";
import { cp, mkdir, writeFile } from "node:fs/promises";
import { readFile } from "node:fs/promises";
import { resolve as pathResolve } from "node:path";

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
                // Ensure an outDir is set for builds (default to ./pkg)
                options.outDir = options.outDir ?? './pkg';
                return buildWasm(options);
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

                const pkgOut = options.outDir || "pkg";
                const pkgDir = pathResolve(process.cwd(), pkgOut);

                // copy entire pkg output into dist/assets so all wasm/js/snippets are present
                const destAssets = pathResolve(distDir, "assets");
                await mkdir(destAssets, { recursive: true });
                try {
                    await cp(pkgDir, destAssets, { recursive: true, force: true });
                } catch (e) {
                    // ignore if pkg doesn't exist
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
                    const pkgEntry = await readFile(pathResolve(pkgDir, 'package.json'), 'utf8')
                        .then(text => JSON.parse(text).main)
                        .catch(() => undefined);
                    const pkgMain = (pkgEntry && assetFiles.includes(pkgEntry) ? pkgEntry : undefined)
                        || assetFiles.find(n => n === 'wasm_bindings.js')
                        || assetFiles.find(n => /^wasm_bindings-.*\.js$/.test(n))
                        || assetFiles.find(n => /^components-.*\.js$/.test(n))
                        || assetFiles.find(n => n === 'components.js')
                        || assetFiles.find(n => /^components_bg.*\.js$/.test(n))
                        || assetFiles.find(n => /^index-.*\.js$/.test(n))
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
                                    content = content.split('import("../../../../..")').join(replacement);
                                    content = content.split("import('../../../../..')").join(replacement);
                                    content = content.replace(/import\(\s*['\"]?\.\.\/\.\.\/\.\.\.\s*['\"]?\s*\)/g, replacement);
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
                    // Duplicate common worker/wasm files to stable filenames to reduce deploy fragility
                    try {
                        const duplicates = [];
                        for (const f of assetFiles) {
                            if (/^worker[-_].*\.js$/.test(f) || /^workerHelpers[-_].*\.js$/i.test(f)) {
                                duplicates.push({ src: f, dest: 'worker.js' });
                            }
                            if (/^worker_helpers[-_].*\.js$/.test(f)) {
                                duplicates.push({ src: f, dest: 'worker_helpers.js' });
                            }
                            if (/^wasm_bindings[-_].*\.js$/.test(f)) {
                                duplicates.push({ src: f, dest: 'wasm_bindings.js' });
                            }
                            if (/^components[-_].*\.js$/.test(f)) {
                                duplicates.push({ src: f, dest: 'components.js' });
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
                    // Create wrapper modules for wasm-imported JS namespaces like `components_bg.js`
                    try {
                        const wasmFiles = assetFiles.filter(n => /_?bg[-_]?.*\.wasm$/.test(n) || /components_bg/.test(n));
                        for (const wasmName of wasmFiles) {
                            // derive base like 'components_bg' from 'components_bg-XXX.wasm' or 'components_bg.wasm'
                            const base = wasmName.replace(/(-[A-Za-z0-9_]+)?\.wasm$/, '').replace(/-bg$/, '_bg');
                            // find a JS candidate that likely provides the glue
                            const jsCandidate = assetFiles.find(n => n.includes(base.replace('_bg', '')) && n.endsWith('.js'))
                                || assetFiles.find(n => n.startsWith(base.replace('_bg', '')) && n.endsWith('.js'))
                                || assetFiles.find(n => /components-.*\.js$/.test(n))
                                || assetFiles.find(n => /wasm_bindings.*\.js$/.test(n));
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