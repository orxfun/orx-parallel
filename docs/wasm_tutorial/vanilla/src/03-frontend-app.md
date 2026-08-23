# The vanilla app

Create the directory for the browser app:

```bash
mkdir app
cd app
```

## Configuration

### `package.json`

Create `par_wasm/app/package.json` as follows:

```json
{
    "name": "orx-parallel-wasm-mini-vanilla-app",
    "version": "0.1.0",
    "private": true,
    "type": "module",
    "scripts": {
        "build:wasm": "ORX_PARALLEL_WASM_BINDINGS=../wasm_bindings ORX_PARALLEL_WASM_OUT_DIR=./pkg node ./node_modules/orx-parallel-wasm/dist/build.js build",
        "dev": "npm exec -- vite",
        "typecheck": "tsc --noEmit",
        "build": "npm run build:wasm && npm run typecheck && npm exec -- vite build"
    },
    "dependencies": {
        "orx-parallel-wasm": "git+https://github.com/orxfun/orx-parallel-wasm.git"
    },
    "devDependencies": {
        "typescript": "^5.6.3",
        "vite": "^5.4.10"
    }
}
```

PLACEHOLDER: explain a bit this package.json

Install the dependencies:

```bash
npm install
```

This creates `package-lock.json` and `node_modules`.

### `tsconfig.json`

Create `par_wasm/app/tsconfig.json` as follows:

```json
{
    "compilerOptions": {
        "target": "ES2020",
        "module": "ESNext",
        "moduleResolution": "Bundler",
        "strict": true,
        "isolatedModules": true,
        "skipLibCheck": true,
        "types": [
            "vite/client"
        ]
    },
    "include": [
        "src"
    ]
}
```

PLACEHOLDER: explain a bit this tsconfig.json, anything special? say nothing special otherwise

### `vite.config.ts`

Create `par_wasm/app/vite.config.ts` as follows:

```ts
import { defineConfig } from "vite";
import { orxParallelWasm } from "orx-parallel-wasm/vite";

export default defineConfig({
    base: "./",
    plugins: [
        orxParallelWasm({
            bindings: "../wasm_bindings",
            outDir: "./pkg",
            threads: 0
        })
    ],
    server: {
        headers: {
            "Cross-Origin-Opener-Policy": "same-origin",
            "Cross-Origin-Embedder-Policy": "require-corp"
        }
    },
    worker: {
        format: "es"
    }
});
```

The plugin compiles the sibling bindings crate and writes generated files to `pkg`.

The two server headers enable `SharedArrayBuffer`, which is required by threaded WebAssembly.

`worker.format` makes the generated worker an ES module.

## Page markup

Create `par_wasm/app/index.html`:

Save this as `index.html`:

```html
<!doctype html>
<html lang="en">

<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>orx-parallel WASM mini tutorial</title>
</head>

<body>
    <main>
        <p class="eyebrow">orx-parallel / WebAssembly</p>
        <h1>Parallel computations using one shared thread pool</h1>
        <p class="intro">Run Rust algorithms with different worker counts and compare their timings.</p>

        <section class="panel" aria-labelledby="settings-title">
            <h2 id="settings-title">Run settings</h2>
            <label>
                Threads
                <input id="threads" type="number" value="0" min="0" step="1" />
                <span id="threads-help">0 uses all initialized threads</span>
            </label>
            <p id="pool-status" role="status">Initializing thread pool...</p>
        </section>

        <section class="computations" aria-label="Computations">
            <article class="computation">
                <p class="index">01</p>
                <h2>Fibonacci workload</h2>
                <p>Sum many Fibonacci terms to give each worker useful CPU work.</p>
                <label>
                    Number of terms
                    <input id="fibonacci-workload" type="number" value="50000" min="1" step="1000" />
                </label>
                <button id="run-fibonacci" type="button">Calculate Fibonacci</button>
                <p id="fibonacci-result" class="result">No result yet.</p>
            </article>

            <article class="computation">
                <p class="index">02</p>
                <h2>Mandelbrot checksum</h2>
                <p>Calculate a checksum across a configurable number of Mandelbrot points.</p>
                <label>
                    Number of points
                    <input id="mandelbrot-workload" type="number" value="50000" min="1" step="1000" />
                </label>
                <button id="run-mandelbrot" type="button">Calculate Checksum</button>
                <p id="mandelbrot-result" class="result">No result yet.</p>
            </article>
        </section>
    </main>
    <script type="module" src="./src/main.ts"></script>
</body>

</html>
```

## Styling

Create `par_wasm/app/style.css`:

```css
:root {
    color: #17221f;
    background: #e8eee8;
    font-family: Georgia, "Times New Roman", serif;
    font-synthesis: none;
}

* {
    box-sizing: border-box;
}

body {
    margin: 0;
    background: linear-gradient(135deg, #e8eee8 0%, #f6f1e8 52%, #d7e4e0 100%);
}

main {
    max-width: 1080px;
    margin: 0 auto;
    padding: 8vh 6vw 10vh;
}

.eyebrow,
.index {
    color: #b34b2d;
    font: 700 0.78rem/1.2 Arial, sans-serif;
    letter-spacing: 0.08em;
    text-transform: uppercase;
}

h1 {
    max-width: 760px;
    margin: 1rem 0;
    font-size: clamp(1.4rem, 3.5vw, 3.25rem);
    line-height: 0.92;
    font-weight: 400;
}

.intro {
    max-width: 560px;
    color: #4d5d57;
    font-size: 1.2rem;
    line-height: 1.5;
}

.panel {
    margin: 4rem 0 2rem;
    padding: 1.5rem;
    border-top: 2px solid #17221f;
    border-bottom: 1px solid #9eaea5;
}

h2 {
    margin: 0.4rem 0 0.7rem;
    font-size: 1.55rem;
    font-weight: 400;
}

label {
    display: grid;
    gap: 0.45rem;
    color: #4d5d57;
    font: 700 0.78rem/1.2 Arial, sans-serif;
    text-transform: uppercase;
}

input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #9eaea5;
    border-radius: 2px;
    color: #17221f;
    background: #fffdf7;
    font: 1rem Georgia, serif;
}

#threads {
    max-width: 12rem;
}

#threads-help,
#pool-status {
    color: #65766e;
    font: 0.85rem Arial, sans-serif;
}

.computations {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1.5rem;
}

.computation {
    padding: 1.6rem;
    border: 1px solid #9eaea5;
    background: rgba(255, 253, 247, 0.72);
}

.computation p:not(.index) {
    color: #65766e;
    line-height: 1.45;
}

.computation label {
    margin: 1.5rem 0;
}

button {
    padding: 0.8rem 1rem;
    border: 0;
    border-radius: 2px;
    color: #fffdf7;
    background: #b34b2d;
    font: 700 0.8rem Arial, sans-serif;
    text-transform: uppercase;
    cursor: pointer;
}

button:hover {
    background: #8f3924;
}

button:disabled {
    cursor: wait;
    opacity: 0.55;
}

.result {
    min-height: 2.8rem;
    margin-bottom: 0;
    font-family: Arial, sans-serif;
}

@media (max-width: 700px) {
    main {
        padding: 2.5rem 1.25rem 5rem;
    }

    .computations {
        grid-template-columns: 1fr;
    }

    h1 {
        font-size: 2rem;
    }
}
```

## TypeScript client

We are ready to create the Typescript client where we will create the thread pool and call exposed parallel computations.

Create `par_wasm/app/src/main.ts`:

```ts
import { ParallelWorker } from "orx-parallel-wasm";
import bindingsUrl from "../pkg/wasm_bindings.js?url";
import "../style.css";

// Desired number of threads in the thread pool, if the hardware allows.
// Setting it to 0 allows using all available threads.
const THREADS_IN_POOL = 0;

type Computations = {
    calculate_fibonacci: (workload: number, threads: number) => bigint;
    mandelbrot_checksum: (limit: number, threads: number) => number;
};

const ui = {
    threads: document.querySelector<HTMLInputElement>("#threads")!,
    threadsHelp: document.querySelector<HTMLSpanElement>("#threads-help")!,
    poolStatus: document.querySelector<HTMLParagraphElement>("#pool-status")!,
    fibonacciWorkload: document.querySelector<HTMLInputElement>("#fibonacci-workload")!,
    mandelbrotWorkload: document.querySelector<HTMLInputElement>("#mandelbrot-workload")!,
    runFibonacci: document.querySelector<HTMLButtonElement>("#run-fibonacci")!,
    runMandelbrot: document.querySelector<HTMLButtonElement>("#run-mandelbrot")!,
    fibonacciResult: document.querySelector<HTMLParagraphElement>("#fibonacci-result")!,
    mandelbrotResult: document.querySelector<HTMLParagraphElement>("#mandelbrot-result")!
};

const worker = new ParallelWorker<Computations>({
    bindingsUrl,
    methods: ["calculate_fibonacci", "mandelbrot_checksum"],
    threads: THREADS_IN_POOL
});

// Per-computation thread limit.
// Setting it to 0 allows using all threads in the thread pool.
function readThreads(): number {
    const value = Number.parseInt(ui.threads.value, 10);
    const maxThreads = worker.initializedThreads ?? 1;
    const threads = Number.isFinite(value) ? Math.max(0, Math.min(maxThreads, value)) : 0;
    ui.threads.value = String(threads);
    return threads;
}

function readPositive(input: HTMLInputElement): number {
    const value = Number.parseInt(input.value, 10);
    return Number.isFinite(value) ? Math.max(1, value) : 1;
}

async function run<T>(button: HTMLButtonElement, output: HTMLParagraphElement, computation: () => Promise<T>): Promise<void> {
    button.disabled = true;
    output.textContent = "Running...";
    const startedAt = performance.now();

    try {
        const result = await computation();
        const elapsed = performance.now() - startedAt;
        output.textContent = `Result: ${String(result)} | ${elapsed.toFixed(2)} ms`;
    } catch (error) {
        output.textContent = `Error: ${error instanceof Error ? error.message : String(error)}`;
    } finally {
        button.disabled = false;
    }
}

void worker.ready().then(
    () => {
        ui.threads.max = String(worker.initializedThreads);
        ui.threadsHelp.textContent = `0 uses all ${worker.initializedThreads} initialized threads`;
        ui.poolStatus.textContent = `Thread pool ready: ${worker.initializedThreads} threads`;
    },
    (error: unknown) => {
        ui.poolStatus.textContent = `Thread pool error: ${error instanceof Error ? error.message : String(error)}`;
        ui.runFibonacci.disabled = true;
        ui.runMandelbrot.disabled = true;
    }
);

ui.runFibonacci.addEventListener("click", () => {
    void run(ui.runFibonacci, ui.fibonacciResult, () =>
        worker.call("calculate_fibonacci", [readPositive(ui.fibonacciWorkload), readThreads()])
    );
});

ui.runMandelbrot.addEventListener("click", () => {
    void run(ui.runMandelbrot, ui.mandelbrotResult, () =>
        worker.call("mandelbrot_checksum", [readPositive(ui.mandelbrotWorkload), readThreads()])
    );
});

window.addEventListener("beforeunload", () => worker.terminate());
```

The generated `pkg/wasm_bindings.js` import is intentionally present before the first build. The `build:wasm` script creates it; after `npm install`, continue with the build instructions in the next chapter.

PLACEHOLDER: mention here how we create the `worker` and how we call the exposed computation functions. they are the important bits.

One level up into `par_wasm` directory:

```bash
cd ..
```
