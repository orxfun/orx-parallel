# Using Transformation

[`ParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html) trait is designed to be as close as possible to the `Iterator` api both due to the fact that:

* iterator api is familiar and awesome, and
* the goal is to parallelize the computation conveniently by changing `iter` with `par`; or `into_iter` with `into_par`.

However, there are certain differences in terms of mutability of the closures, mainly to prevent race conditions.

## The limitation

For instance, the following is the `map` signature of the sequential `Iterator`:

```rust
fn map<B, F>(self, f: F) -> Map<Self, F>
where
    F: FnMut(Self::Item) -> B;
```

while, the map signature of the parallel `ParIter` uses `Fn` rather than `FnMut`:

```rust
fn map<B, F>(self, f: F) -> impl ParIter<R, Item = B>
where
    F: Fn(Self::Item) -> B;
```

It might be obvious but to clarify why parallel map can only accept `Fn`, assume that `f` captures a mutable counter, and increments the counter every time it is called.

* This is perfectly fine in a sequential iterator. The counter will be incremented one at a time.
* However, in parallel computation, multiple threads will be trying to increment this mutable counter at the same time leading to a race condition. Limiting `f` to be `Fn`, we guarantee that our computation is free of race condition.

## A simple example

Consider that we have a computation that requires random numbers. For brevity, random number generators can be considered as iterators returning a series of random numbers as we request. Therefore, it keeps its current position and generates random numbers via a mutable reference.

Consider the following example, where we take a set of `positions` and take 100 random steps to left or to right starting from each one of them. Then, we check the final positions.

In order to take the random step, we need a mutable `rng`.

Since the sequential iterator's `map` method allows for `FnMut`:
* we can use `.map(|position| random_walk(&mut rng, position, 100))`,
* which matches the signature `F: FnMut(i64) -> i64`,
* while the captured mutable `rng` reference is the reason of the closure being `FnMut` and it is conveniently abstracted away.

```rust
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn random_walk(rng: &mut impl Rng, position: i64, num_steps: usize) -> i64 {
    (0..num_steps).fold(position, |p, _| random_step(rng, p))
}

fn random_step(rng: &mut impl Rng, position: i64) -> i64 {
    match rng.random_bool(0.5) {
        true => position + 1,  // to right
        false => position - 1, // to left
    }
}

fn input_positions() -> Vec<i64> {
    (-10_000..=10_000).collect()
}

fn sequential() {
    let positions = input_positions();
    let sum_initial_positions = positions.iter().sum::<i64>();
    println!("sum_initial_positions = {sum_initial_positions}");

    let mut rng = ChaCha20Rng::seed_from_u64(42);
    let final_positions: Vec<_> = positions
        .iter()
        .copied()
        .map(|position| random_walk(&mut rng, position, 100))
        .collect();
    let sum_final_positions = final_positions.iter().sum::<i64>();
    println!("sum_final_positions = {sum_final_positions}");
}
```

However, we cannot do the same with a parallel iterator. The following will not compile:

```rust
fn parallel() {
    let positions = input_positions();
    let sum_initial_positions = positions.iter().sum::<i64>();
    println!("sum_initial_positions = {sum_initial_positions}");

    let mut rng = ChaCha20Rng::seed_from_u64(42);
    let final_positions: Vec<_> = positions
        .par() // <-- parallel computation
        .copied()
        .map(|position| random_walk(&mut rng, position, 100)) // <-- does not compile!!
        .collect();
    let sum_final_positions = final_positions.iter().sum::<i64>();
    println!("sum_final_positions = {sum_final_positions}");
}
```

And it should not compile. If it did, multiple threads would call `random_bool` on the same `rng` at the same time, which would lead to the race condition.

## Solution `using` an explicit mutable variable

[`ParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html) does not allow the parallel computation defined above; however, [`ParIterUsing`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIterUsing.html) enables it safely with the following approach:

* No race conditions are allowed; therefore, there cannot be one mutable **variable** that all threads accesses.
* Instead, all threads have their own mutable **variable**.
* Since the computation within the thread is sequential, there cannot be any race condition and we can freely mutate the **variable**.
* This **variable** is explicitly defined by one of the two methods which transforms the `ParIter` into `ParIterUsing`.
  * [`using`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html#tymethod.using): method takes a closure with signature `F: FnMut(usize) -> U`. It takes the index of the spawned thread as input and creates an instance of the variable of type `U`.
  * [`using_clone`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html#tymethod.using_clone) instead takes a cloneable value of type `U` and provides one clone of it to each of the threads. We can consider `par.using_clone(value)` as a shorthand for `par.using(|_thread_idx| value.clone())`.

Provided that the parallel computation is executed with `N` threads, then **exactly** `N` different instances of `U` will be created and send to each thread.

Then, a mutable reference to this variable will be available to all of the parallel iterator methods. For instance, the signature of `map` method of the `ParIterUsing` is as follows:

```rust
fn map<U, B, F>(self, f: F) -> impl ParIter<R, Item = B>
where
    F: Fn(&mut U, Self::Item) -> B;
```

Notice that the closure is safely `Fn` as it does not mutate any captured variable. On the other hand, it explicitly takes a mutable reference to a value of `U`. This allows us to represent the computation above and execute it in parallel as follows:

```rust
fn parallel() {
    let positions = input_positions();
    let sum_initial_positions = positions.iter().sum::<i64>();
    println!("sum_initial_positions = {sum_initial_positions}");

    let final_positions: Vec<_> = positions
        .par() // <-- parallel computation
        .copied()
        .using(|t_idx| ChaCha20Rng::seed_from_u64(42 * t_idx as u64)) // <-- explicit using
        .map(|rng, position| random_walk(rng, position, 100)) // <-- safe access to mutable rng
        .collect();
    let sum_final_positions = final_positions.iter().sum::<i64>();
    println!("sum_final_positions = {sum_final_positions}");
}
```

There are two important differences here.

1. With the following line, we are expressing that each thread will create a random number generator (rng). Further, we state that the seed of the rng will be `42 * t_idx` which guarantees that no two threads will use the same sequence of random numbers (important when it matters).

```rust
    .using(|t_idx| ChaCha20Rng::seed_from_u64(42 * t_idx as u64))
```

2. Next, in the `map` call, we have access to a mutable reference to the used variable. This variable is the `rng` created for that specific thread.

```rust
    .map(|rng, position| random_walk(rng, position, 100)) // rng: &mut ChaCha20Rng
```

## Generalization

Notice that once the safety measures are defined by `ParIterUsing`, it is not different to implement `map`, `filter` or `for_each`, etc. Therefore, all these methods have access to a mutable reference of `U`. The following example demonstrates some of them, all with safe access to the mutable variable `rng`.

```rust
let input: Vec<u64> = (1..N).collect();
let some_counter = AtomicU64::new(0);

input
    .into_par()
    .using(|thread_idx| ChaCha20Rng::seed_from_u64(thread_idx as u64 * 10))
    .map(|_, i| fibonacci((i % 50) + 1) % 100)
    .filter(|rng, _| rng.random_bool(0.4))
    .flat_map(|rng, i| [rng.random_range(0..i), rng.random_range(0..i)])
    .inspect(|rng, i| {
        if *i < 42 && rng.random_bool(0.2) {
            some_counter.fetch_add(*i, Ordering::Relaxed);
        }
    })
    .sum()
```

## Examples - Channels

Random number generator is one of the common use cases that is important for a certain class of algorithms.

However, there are other use cases where access to mutable variable is useful. `rayon` allows such computations with `map_with` and `for_each_with` methods, and channels are used as examples in the corresponding documentations. The following example is taken from these documentations and converted to `using` transformation:

```rust
use orx_parallel::*;
use std::sync::mpsc::channel;

let (sender, receiver) = channel();

(0..5)
    .into_par()
    .using_clone(sender)
    .for_each(|s, x| s.send(x).unwrap());

let mut res: Vec<_> = receiver.iter().collect();

res.sort();

assert_eq!(&res[..], &[0, 1, 2, 3, 4])
```

## Examples - Metrics

Another potential use case is to be able to collect certain metrics about the parallel execution, which is often not trivial.

Revisiting the safety notes above, we should be able to collect metrics through mutation per each thread which would give us insights about the parallel execution. To achieve this, in addition to `using`, we need some `unsafe` help with interior mutability.

You may see the corresponding example file here: [using_metrics](https://github.com/orxfun/orx-parallel/blob/main/examples/using_metrics.rs).

```rust
use orx_parallel::*;
use std::cell::UnsafeCell;

const N: u64 = 10_000_000;
const MAX_NUM_THREADS: usize = 8;

// just some work
fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

#[derive(Default, Debug)]
struct ThreadMetrics {
    thread_idx: usize,
    num_items_handled: usize,
    handled_42: bool,
    num_filtered_out: usize,
}

struct ThreadMetricsWriter<'a> {
    metrics_ref: &'a mut ThreadMetrics,
}

struct ComputationMetrics {
    thread_metrics: UnsafeCell<[ThreadMetrics; MAX_NUM_THREADS]>,
}
impl ComputationMetrics {
    fn new() -> Self {
        let mut thread_metrics: [ThreadMetrics; MAX_NUM_THREADS] = Default::default();
        for i in 0..MAX_NUM_THREADS {
            thread_metrics[i].thread_idx = i;
        }
        Self {
            thread_metrics: UnsafeCell::new(thread_metrics),
        }
    }
}

impl ComputationMetrics {
    unsafe fn create_for_thread<'a>(&mut self, thread_idx: usize) -> ThreadMetricsWriter<'a> {
        // SAFETY: here we create a mutable variable to the thread_idx-th metrics
        // * If we call this method multiple times with the same index,
        //   we create multiple mutable references to the same ThreadMetrics,
        //   which would lead to a race condition.
        // * We must make sure that `create_for_thread` is called only once per thread.
        // * If we use `create_for_thread` within the `using` call to create mutable values
        //   used by the threads, we are certain that the parallel computation
        //   will only call this method once per thread; hence, it will not
        //   cause the race condition.
        // * On the other hand, we must ensure that we do not call this method
        //   externally.
        let array = unsafe { &mut *self.thread_metrics.get() };
        ThreadMetricsWriter {
            metrics_ref: &mut array[thread_idx],
        }
    }
}

fn main() {
    let mut metrics = ComputationMetrics::new();

    let input: Vec<u64> = (0..N).collect();

    let sum = input
        .par()
        // SAFETY: we do not call `create_for_thread` externally;
        // it is safe if it is called only by the parallel computation.
        .using(|t| unsafe { metrics.create_for_thread(t) })
        .map(|m: &mut ThreadMetricsWriter<'_>, i| {
            // collect some useful metrics
            m.metrics_ref.num_items_handled += 1;
            m.metrics_ref.handled_42 |= *i == 42;

            // actual work
            fibonacci((*i % 50) + 1) % 100
        })
        .filter(|m, i| {
            let is_even = i % 2 == 0;

            if !is_even {
                m.metrics_ref.num_filtered_out += 1;
            }

            is_even
        })
        .num_threads(MAX_NUM_THREADS)
        .sum();

    println!("\nINPUT-LEN = {N}");
    println!("SUM = {sum}");

    println!("\n\n");

    println!("COLLECTED METRICS PER THREAD");
    for metrics in metrics.thread_metrics.get_mut().iter() {
        println!("* {metrics:?}");
    }
    let total_by_metrics: usize = metrics
        .thread_metrics
        .get_mut()
        .iter()
        .map(|x| x.num_items_handled)
        .sum();
    println!("\n-> total num_items_handled by collected metrics: {total_by_metrics:?}\n");

    assert_eq!(N as usize, total_by_metrics);
}
```

Note that creating a thread metrics writer with the `ComputationMetrics::create_for_thread` method is `unsafe` as we can create multiple of them for the same thread metrics. However, parallel execution will not do this: it guarantees that the method will be called once per thread; hence, each call with a different thread index.

Once we define how to create the thread metrics writer with `using(|t| unsafe { metrics.create_for_thread(t) })`, the writer is then conveniently available to each of the parallel iterator methods. We can  safely use it within each of the closures to collect information; or simply omit when not required.

At the end of the computation, the collected metrics will be available by the computation metrics, `metrics`.

The output of the program is as follows:

```bash
INPUT-LEN = 10000000
SUM = 162400000

COLLECTED METRICS PER THREAD
* ThreadMetrics { thread_idx: 0, num_items_handled: 1310720, handled_42: true, num_filtered_out: 891288 }
* ThreadMetrics { thread_idx: 1, num_items_handled: 1310720, handled_42: false, num_filtered_out: 891290 }
* ThreadMetrics { thread_idx: 2, num_items_handled: 1310720, handled_42: false, num_filtered_out: 891290 }
* ThreadMetrics { thread_idx: 3, num_items_handled: 1310720, handled_42: false, num_filtered_out: 891290 }
* ThreadMetrics { thread_idx: 4, num_items_handled: 1087104, handled_42: false, num_filtered_out: 739231 }
* ThreadMetrics { thread_idx: 5, num_items_handled: 1310720, handled_42: false, num_filtered_out: 891290 }
* ThreadMetrics { thread_idx: 6, num_items_handled: 1048576, handled_42: false, num_filtered_out: 713032 }
* ThreadMetrics { thread_idx: 7, num_items_handled: 1310720, handled_42: false, num_filtered_out: 891289 }

-> total num_items_handled by collected metrics: 10000000
```
