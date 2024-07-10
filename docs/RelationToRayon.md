# orx-parallel

## Relation to `rayon`

[https://github.com/rayon-rs/rayon](https://github.com/rayon-rs/rayon)

> Rayon is a data-parallelism library for Rust. It is extremely lightweight and makes it easy to convert a sequential computation into a parallel one. It also guarantees data-race freedom. (You may also enjoy [this blog post][blog] about Rayon, which gives more background and details about how it works, or [this video][video], from the Rust Belt Rust conference.) Rayon is [available on crates.io](https://crates.io/crates/rayon), and [API documentation is available on docs.rs](https://docs.rs/rayon).

[blog]: https://smallcultfollowing.com/babysteps/blog/2015/12/18/rayon-data-parallelism-in-rust/
[video]: https://www.youtube.com/watch?v=gof_OEv71Aw

Rayon is a very mature library allowing us to execute complex tasks in parallel since I do rust.

Then why another crate? Because it is different and simple -> that is fun & exciting.

### Usage

Defining parallel computation through the iterator methods is almost identical in this crate and in rayon. Just as in regular iterators we love, this is an ergonomic, composable and less error-prone way to represent the computation.

### Underlying Approach

Underlying approaches, however, are quite different.

#### |> rayon

Please refer to the blog mentioned above or to the repository for details. To the best of my understanding, rayon utilizes the primitive `join` method. In the following example, the two tasks are to be executed before the method returns. It is very elegant to build on this simple primitive. Further, its recursive nature allows it to be used, I believe, anywhere.

```rust
join(|| do_something(), || do_something_else())
```

These two tasks will *potentially* be executed in parallel. Depending on the available cores and workload, the runtime decides how to execute these two tasks: in parallel or not. With a smart runtime, this *potential parallelism* approach turns out to be very efficient.

#### |> orx-parallel

`Par` defined in this crate is built on top of [`ConcurrentIter`](https://crates.io/crates/orx-concurrent-iter) which does nothing but allows to iterate over its elements concurrently from multiple threads.

This turns out to be sufficient to build a parallel executor that is arguably as simple as it could be:
* it spawns threads,
* each thread pulls tasks from the iterator, executes them, and pulls more from the remaining tasks whenever they are idle again,
* the computation returns once the iterator is consumed (or an early exit condition is satisfied as in `find` method).

There exist only two straightforward decisions to take:
* how many threads to spawn?
* how many tasks to pull each time a thread is idle?

The caller can have complete control on these two settings. This is particularly useful for tuning or for putting limits on resource usage per computation.

Alternatively, the caller can leave the decisions completely to the heuristic implemented in this crate. Current heuristic has been a quick implementation without much thought on it yet; however, it achieves a high performance computation. This is possible due to the simplicity of the approach. To demonstrate this, consider the following parallel map implementation, which has been the starting point of this crate. It is a little simplified version of the current implementation; however, not really far from it. 

```rust
use orx_concurrent_ordered_bag::*;
use orx_concurrent_iter::*;

fn map(input: u64) -> String {
    input.to_string()
}

fn parallel_map(
    num_threads: usize,
    chunk_size: usize,
    iter: impl ConcurrentIter<Item = u64>,
) -> SplitVec<String> {
    let outputs = ConcurrentOrderedBag::new();
    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| match chunk_size {
                1 => {
                    iter.ids_and_values()
                        .map(|(idx, value)| (idx, map(value)))
                        .for_each(|(idx, value)| unsafe { outputs.set_value(idx, value) });
                }
                c => {
                    while let Some(chunk) = iter.next_chunk(c) {
                        let begin_idx = chunk.begin_idx;
                        unsafe { outputs.set_values(begin_idx, chunk.values.map(&map)) };
                    }
                }
            });
        }
    });
    unsafe { outputs.into_inner().unwrap() }
}
```

### Performance

Benchmarks are tricky, even trickier in parallel context. At least in most of the [benchmarks](https://github.com/orxfun/orx-parallel/blob/main/benches) defined in this crate, we observe that rayon and orx-parallel perform comparably.

Using concurrent collectors such as [`ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) or [`ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag), `Par` is particularly efficient in scenarios where the results are collected, the improvement is most significant in the [`flat_map`](https://github.com/orxfun/orx-parallel/blob/main/benches/flatmap.rs) examples.
