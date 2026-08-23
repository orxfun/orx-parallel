# The vanilla app

Create a worker-backed client with the package:

```ts
const worker = new ParallelWorker<Computations>({
    bindingsUrl,
    methods: ["calculate_fibonacci", "mandelbrot_checksum"],
    threads: 0
});
```

`bindingsUrl` points at the generated `wasm_bindings.js` module. After `worker.ready()` resolves, `worker.initializedThreads` gives the capacity used to cap the number input.

Each button records `performance.now()` before calling `worker.call(...)`. The result and elapsed milliseconds are then written to its output element. Both buttons share the pool, but each has its own workload input so the Fibonacci and Mandelbrot computations can be tuned independently. The thread input is clamped to the initialized pool capacity before each call; `0` retains the library’s all-threads behavior.
