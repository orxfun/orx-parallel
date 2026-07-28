# orx-parallel

[![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel crate](https://img.shields.io/crates/d/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)

High-performance, configurable, expressive parallel iterators for Rust.

`orx-parallel` keeps the iterator style you already know while giving you:

- strong performance on many practical workloads,
- per-computation control over thread count,
- broad collection coverage (including arbitrary iterators),
- first-class fallible and optional parallel flows,
- explicit runner/pool separation for advanced scheduling control.

## Why This Crate

The goal is simple: parallelize existing iterator pipelines with minimal friction.

In many places, the change is only:

- `iter()` -> `par()`
- `into_iter()` -> `into_par()`

```rust
use orx_parallel::*;

let sum: usize = (0..1_000_000)
	.par()
	.map(|x| x * 2)
	.filter(|x| x % 3 == 0)
	.sum();

assert!(sum > 0);
```

## Rayon and orx-parallel

`rayon` is excellent and battle-tested. This crate is not a replacement message; it is an additional option with different design choices.

Pick `rayon` when:

- you want the familiar Rayon ecosystem and defaults,
- Rayon already matches your workload and architecture needs.

Pick `orx-parallel` when:

- you want explicit per-computation control like `.num_threads(...)`,
- you need fallible or optional pipelines with short-circuiting via `into_fallible` / `into_optional`,
- you want to tune execution by swapping runners and pools, including custom implementations,
- you want to parallelize arbitrary iterators with `iter_into_par()` in addition to collection-native paths.

## What Stands Out

### 1) Performance on real iterator pipelines

Benchmarks are in [benches](benches) with captured outputs such as:

- [benches/collect/run_results.txt](benches/collect/run_results.txt)
- [benches/reduce/run_results.txt](benches/reduce/run_results.txt)
- [benches/first/run_results.txt](benches/first/run_results.txt)
- [benches/het/run_results.txt](benches/het/run_results.txt)
- [benches/rec/run_results.txt](benches/rec/run_results.txt)

Sample results from those files (same machine/run set):

- `reduce_mf` (heavy, n=2e20): `orx = 6,636,929 ns`, `rayon = 17,711,519 ns` (~2.67x faster for `orx`)
- `first_f` (heavy-mid, n=2e20, nt=32): `orx = 6,860,511 ns`, `rayon = 24,283,885 ns` (~3.54x faster for `orx`)
- `col_l` (heavy, n=2e20): `orx-arb-vec2 = 18,013,458 ns`, `rayon-vec = 52,945,777 ns` (~2.94x faster for `orx`)

As usual with parallel workloads, outcomes depend on workload shape and scheduling. Some cases favor Rayon; many measured scenarios here favor `orx-parallel`.

### 2) API convenience close to `Iterator`

`Par`/`IntoPar` transformations are designed to feel like sequential iterators while remaining race-safe.

See examples:

- [examples/enumerate.rs](examples/enumerate.rs)
- [examples/par_mut.rs](examples/par_mut.rs)

### 3) Coverage: optimized common collections + arbitrary iterators

Direct optimized paths exist for common inputs such as `Vec`, slices, `VecDeque`, and ranges.

Any regular iterator can also be parallelized via `iter_into_par()`:

```rust
use orx_parallel::*;
use std::collections::HashMap;

let mut map: HashMap<_, _> = (0..1024).map(|x| (x.to_string(), x)).collect();

map.values_mut()
	.iter_into_par()
	.filter(|x| **x % 2 == 0)
	.for_each(|x| *x *= 2);
```

This means practically any collection can be parallelized through its iterators. If you need a specialized direct implementation for a collection, opening an issue is very welcome.

### 4) Fallible and optional iterators are first-class

Use explicit transformations and keep writing the success path:

- `into_fallible()` for `Result` item flows
- `into_optional()` for `Option` item flows

Examples:

- [examples/fallible_result.rs](examples/fallible_result.rs)
- [examples/fallible_option.rs](examples/fallible_option.rs)

### 5) Per-computation thread configuration

Thread count is configurable at computation level with `.num_threads(...)` and coexists with pool/environment limits.

This is useful when:

- multiple parallel computations run concurrently,
- one stage should leave resources for other services/tasks,
- you want predictable throughput/latency trade-offs per workload.

Details:

- [docs/threading_model.md](docs/threading_model.md)

### 6) Runner and iterator are decoupled

Parallel iterator definition and execution strategy are separate concerns:

- express computation with iterator transforms,
- choose runner strategy (`fixed`) independently.

This enables advanced users to implement domain-specific runners and use them with `.runner(...)`.

### 7) Any thread pool can be integrated

Pools are abstracted behind `ParThreadPool`. You can:

- use built-ins via `Pool::once(...)` or `Pool::basic(...)`,
- integrate external pools (for example Rayon pool via `Pool::rayon(...)` with `rayon-core` feature),
- implement `ParThreadPool` for your own pool.

### 8) Expressive recursive parallel iterators

Recursive/nonlinear traversals are ergonomic with `into_par_recursive(...)`.

Example:

- [examples/recursive.rs](examples/recursive.rs)

### 9) Diagnostics for load distribution

Use `runner_with_diagnostics()` to inspect how work was distributed across threads and tune accordingly.

Example:

- [examples/diagnostics.rs](examples/diagnostics.rs)

## Install

```toml
[dependencies]
orx-parallel = "3.4"
```

Optional features for advanced scenarios:

- `rayon-core` for direct Rayon thread pool integration,
- `wasm` for web wasm threading integration (requires wasm thread toolchain/runtime setup),
- `std` (enabled by default).

## More Documentation

- API docs: <https://docs.rs/orx-parallel>
- Threading model: [docs/threading_model.md](docs/threading_model.md)
- WebAssembly usage, setup, and troubleshooting: [docs/wasm.md](docs/wasm.md)
- WebAssembly internals and backend details: [docs/wasm_internals.md](docs/wasm_internals.md)
- Using mutable per-thread state safely: [docs/using.md](docs/using.md)
- Examples folder: [examples](examples)
- Bench suite: [benches](benches)

## Contributing

Contributions are very welcome.

Particularly valuable areas:

- runner design and scheduling for challenging workload shapes,
- collection-specific optimized parallelization paths,
- benchmark scenarios and reproducible performance reports,
- documentation and examples.

If you are experienced in thread scheduling, pool internals, or workload balancing, your input can directly shape the crate's next major gains.

