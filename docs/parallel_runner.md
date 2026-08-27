# Parallel Runner Guide

This guide describes the `ParRunner` API and gives a concrete path for implementing an experimental parallel runner.

A parallel runner controls how work is distributed across threads. It does not own the input, the iterator transformations, or the thread pool itself. Those responsibilities are intentionally separated:

- `ConcurrentIter` owns concurrent access to input items.
- `Xap` variants own iterator transformations such as `map`, `filter`, and `flat_map`.
- `ParThreadPool` owns scoped execution and worker lifecycle.
- `ParRunner` owns scheduling decisions: how many workers to start and how large each pulled chunk should be.

This separation is why a new runner can usually be implemented without changing iterator APIs or collection internals.

## Where runners live

Current built-in runners are under:

- `src/runner/runner_variants/fixed_chunk`
- `src/runner/runner_variants/adaptive_chunk`

The public factory methods are in:

- `src/runner/new_runner.rs`

The default runner is selected in:

- `src/runner/mod.rs`

With the `std` feature, the default is currently `AdaptiveChunkRunner<DefaultPool>`. Without `std`, the default is `FixedChunkRunner<DefaultPool>`.

## The `ParRunner` contract

The trait is defined in `src/runner/par_runner.rs`:

```rust ignore
pub trait ParRunner: Sized + Sync {
    type Pool: ParThreadPool;
    type State: Send + Sync;
    type ChunkState;

    fn pool(&self) -> &Self::Pool;
    fn pool_mut(&mut self) -> &mut Self::Pool;

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize>;

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        size_hint: (usize, Option<usize>),
    ) -> Self::State;

    fn begin_thread(state: &Self::State, th_idx: usize);
    fn next_chunk_size(state: &Self::State, size_hint: (usize, Option<usize>)) -> usize;
    fn begin_chunk(th_idx: usize, chunk_size: usize) -> Self::ChunkState;
    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState);
    fn complete_thread(state: &Self::State, th_idx: usize);
    fn complete_computation(state: Self::State);
}
```

The runner must be `Sync` because shared runner state is read from worker tasks. If the runner stores a pool that is safe to use in the crate's execution model but does not automatically implement `Sync`, the built-in runners use an explicit unsafe `Sync` implementation. New runners should only do the same after checking that all shared mutation is inside `State` and protected by atomics, locks, or thread-local ownership.

## Lifecycle of a computation

Most terminal operations follow the same shape. The exact code lives in modules such as `src/infallible/par_runner.rs`, `src/option/par_runner.rs`, and `src/result/par_runner.rs`.

1. The iterator pipeline reaches a terminal operation such as `collect`, `reduce`, `find`, `all`, or `for_each`.
2. The execution layer calls `runner.nt_state(params, iter.size_hint(), computation_max_nt)`.
3. `nt_state` asks the pool for the maximum usable thread count with `pool.max_num_threads_for_computation(params, size_hint)`.
4. `nt_state` calls `new_state(...)` to create one shared state value for this computation.
5. The execution layer enters `pool_mut().scoped_computation(...)`.
6. Inside that scope, `do_spawn_new(spawned, &state)` is called sequentially until it returns `None`.
7. For every returned thread index, the pool starts one scoped worker task.
8. Each worker calls `begin_thread(&state, th_idx)` once.
9. Worker execution loops call `next_chunk_size(&state, iter.size_hint())` repeatedly.
10. For each requested chunk, the worker calls `begin_chunk(th_idx, chunk_size)` before pulling work.
11. After the chunk is handled, the worker calls `complete_chunk(&state, chunk_state)`.
12. When the worker finishes, it calls `complete_thread(&state, th_idx)`.
13. After the scoped computation completes, the execution layer calls `complete_computation(state)`.

A runner therefore sees the computation only through thread counts, size hints, chunk boundaries, and lifecycle callbacks. It should not know about `map`, `filter`, `collect`, or the concrete input collection.

## Thread-count decisions

A runner does not choose the pool capacity by itself. The helper `nt_state` combines:

- pool capacity from `ParThreadPool::max_num_threads()`
- `ORX_NUM_THREADS`, already reflected in the pool capacity
- per-computation `.num_threads(...)`
- input-size upper bound when it is known

The `max_num_threads` passed to `new_state(...)` is the maximum number of worker tasks the runner should use for that computation. A runner can spawn fewer, but it must not return a thread index greater than or equal to `max_num_threads` from `do_spawn_new`.

The built-in runners currently spawn all allowed workers:

```rust ignore
fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
    (spawned < state.max_num_threads).then_some(spawned)
}
```

An experimental runner may choose to spawn fewer workers, but it should be benchmarked carefully. Under-spawning can help small or memory-bound tasks, but it can also leave CPU-bound workloads underutilized.

## Chunk-size decisions

The most important runner hook is usually:

```rust ignore
fn next_chunk_size(state: &Self::State, size_hint: (usize, Option<usize>)) -> usize;
```

This method is called by worker loops before pulling from the `ConcurrentIter`. A larger chunk amortizes atomic input-pulling overhead but can hurt load balance. A smaller chunk improves balancing and early-exit responsiveness but increases overhead.

Existing strategies:

- `FixedChunkRunner` computes one chunk size in `new_state(...)` and always returns it.
- `AdaptiveChunkRunner` starts with small chunks, records timing in `complete_chunk(...)`, and then switches to a selected fixed chunk size.

Use `Params::chunk_size` correctly:

- `ChunkSize::Exact(n)` means the runner should use exactly `n` where possible.
- `ChunkSize::Min(n)` means adaptive strategies should not go below `n`.
- `ChunkSize::Auto` leaves the decision to the runner.

## `State` and `ChunkState`

`State` is shared across worker tasks. It should hold computation-level scheduling data:

- maximum usable thread count
- selected or minimum chunk size
- timing aggregates
- per-thread counters
- cancellation or exploration flags

Use atomics or synchronization for any state mutated from workers. Avoid storing references to input values or transformation closures in the runner state; those belong to the execution layer.

`ChunkState` is returned by `begin_chunk(...)` and later passed to `complete_chunk(...)`. Use it for data that belongs to one chunk execution:

- start time
- requested chunk size
- thread index
- local counters

If the runner does not need per-chunk feedback, set `type ChunkState = ()`.

## Minimal experimental runner

A simple starting point is a fixed-size runner with a custom chunk heuristic. This can live under a new module such as `src/runner/runner_variants/my_runner` while it is being tested.

```rust ignore
use orx_parallel::{ChunkSize, NumThreads, Params, ParThreadPool};
use orx_parallel::Runner;
use orx_parallel::runner::ParRunner; // if the trait is exported through your branch/module layout

pub struct MyRunner<P: ParThreadPool> {
    pool: P,
}

pub struct MyState {
    max_num_threads: usize,
    chunk_size: usize,
}

impl<P: ParThreadPool> MyRunner<P> {
    pub fn new(pool: P) -> Self {
        Self { pool }
    }
}

unsafe impl<P: ParThreadPool> Sync for MyRunner<P> {}

impl<P: ParThreadPool> ParRunner for MyRunner<P> {
    type Pool = P;
    type State = MyState;
    type ChunkState = ();

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.pool
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        (spawned < state.max_num_threads).then_some(spawned)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        size_hint: (usize, Option<usize>),
    ) -> Self::State {
        let chunk_size = match params.chunk_size {
            ChunkSize::Exact(n) | ChunkSize::Min(n) => n.into(),
            ChunkSize::Auto => match size_hint.1 {
                Some(len) => (len / (max_num_threads * 8)).max(1),
                None => 1,
            },
        };

        MyState {
            max_num_threads,
            chunk_size,
        }
    }

    fn begin_thread(_: &Self::State, _: usize) {}

    fn next_chunk_size(state: &Self::State, _: (usize, Option<usize>)) -> usize {
        state.chunk_size
    }

    fn begin_chunk(_: usize, _: usize) -> Self::ChunkState {}

    fn complete_chunk(_: &Self::State, _: Self::ChunkState) {}

    fn complete_thread(_: &Self::State, _: usize) {}

    fn complete_computation(_: Self::State) {}
}
```

Then use it with a custom pool:

```rust ignore
use orx_parallel::*;

let pool = Pool::basic(8);
let runner = MyRunner::new(pool);
let sum: usize = (0..1_000_000).par().runner(runner).sum();
```

During early experimentation, it is often easier to add a temporary factory method in `src/runner/new_runner.rs`:

```rust ignore
impl Runner {
    pub fn my_runner_with_pool<P: ParThreadPool>(pool: P) -> MyRunner<P> {
        MyRunner::new(pool)
    }
}
```

## Making a runner the default on a branch

To benchmark an experimental runner as the default runner, change `src/runner/mod.rs` on your branch.

For example, the default runner aliases currently look like this:

```rust ignore
#[cfg(not(feature = "std"))]
pub type DefaultRunner = runner_variants::FixedChunkRunner<crate::pool::DefaultPool>;

#[cfg(feature = "std")]
pub type DefaultRunner = runner_variants::AdaptiveChunkRunner<crate::pool::DefaultPool>;

pub fn default_runner() -> DefaultRunner {
    DefaultRunner::new(get_global_pool())
}
```

For a new runner, expose it from `src/runner/runner_variants/mod.rs`, then point the relevant `DefaultRunner` alias at it. Keep `DefaultRunner::new(get_global_pool())` working, or update `default_runner()` if the constructor needs additional configuration.

## Testing checklist

Run the normal test suite first:

```sh
cargo test
cargo test --doc
```

Then test the runner across execution modes and operations:

- `Runner::fixed()` and `Runner::adaptive()` should remain unchanged.
- `Runner::{fixed,adaptive}_with_pool(...)` should still work with `Pool::basic`, `Pool::once`, and the Rayon-backed pool when enabled.
- Ordered operations should produce the same result with `num_threads(1)` and `num_threads(N)`.
- `collect`, `first`, `find`, `all`, `any`, `reduce`, `fold`, and `for_each` should all be exercised.
- Option and result flows should still short-circuit correctly.
- `use_new`, `use_vec`, and `use_slice` should still provide one mutable state value per participating worker.
- Recursive computations through `par_recursive(...)` should remain deterministic in ordered mode.

Useful test locations include:

- `tests/`
- `src/infallible/tests`
- `src/option/tests`
- `src/result/tests`
- `src/infallible_use/tests`
- `src/option_use/tests`
- `src/result_use/tests`
- `src/infallible/recursive/execution/tests`

## Benchmarking workflow

Use [`orx-parallel-benchmarks`](https://github.com/orxfun/orx-parallel-benchmarks) for performance checks. The benchmark repository is designed so each benchmark runs in a separate process, which avoids interference from persistent thread pools.

Recommended workflow:

1. Create a branch in `orx-parallel` with the experimental runner.
2. If needed, make the runner the default in `src/runner/mod.rs` on that branch.
3. In `orx-parallel-benchmarks`, point the relevant benchmark `Cargo.toml` files at your branch or local path.
4. Run benchmarks for at least these categories:
   - uniform CPU-heavy map/reduce
   - heterogeneous work
   - memory-bound map/filter/collect
   - early-exit search
   - fallible option/result flows
   - recursive traversal
5. Compare against both built-in runners and Rayon where the benchmark has a Rayon variant.

A runner is promising only if it improves some workload without causing unacceptable regressions elsewhere. Prefer collecting enough data to describe where the runner should be used instead of trying to find one strategy that wins everywhere.

## Design advice

- Keep the iterator API unchanged while experimenting with runners.
- Treat `Params` as user intent; respect exact chunk sizes and thread-count caps.
- Use `size_hint` when it is available, but keep unknown-length inputs efficient.
- Optimize for predictable behavior first, then for speed.
- Make ordered and arbitrary behavior explicit in tests.
- Prefer structured diagnostics over println-only debugging.
- Keep runner-specific state in the runner module; do not leak scheduling details into `Par`, `ParOption`, `ParResult`, or collection code.
- Add benchmarks before making a new runner the default.
