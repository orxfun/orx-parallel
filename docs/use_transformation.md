# Use Transformation with `orx-parallel`

`use` transformations provide a safe and ergonomic way to use mutable worker-local state in parallel pipelines.

Highlights:

- convenience and safety: no unsafe code in application-level iterator logic
- memory efficiency: exactly one use-variable per worker thread
- predictable allocation behavior for stateful workloads

## `use_new`: Random number generators, a very common use case

> `use_new` creates one variable per thread and makes its mutable reference available to thread-local computations. Created *use variables* are dropped once the computation finalizes.

Recall the entry example in the library which demonstrates parallelization with iterator api:

```rust
use orx_parallel::*;
use rand::prelude::*;

struct Tour(Vec<usize>);

impl Tour {
    fn random(n: usize) -> Self {
        let mut cities: Vec<_> = (0..n).collect();
        cities.shuffle(&mut rand::rng());
        Self(cities)
    }

    fn starts_at_coffee_shop(&self) -> bool {
        self.0.first() == Some(&7)
    }

    fn duration(&self) -> u64 {
        let links = self.0.iter().zip(self.0.iter().skip(1));
        links
            .map(|(a, b)| (*a as i64 - *b as i64).unsigned_abs())
            .sum::<u64>()
    }
}

let num_tours = 1_000_000;
let num_cities = 10;

// parallel
let best_tour = (0..num_tours)
    .par() // ← parallelized
    .map(|_| Tour::random(num_cities))
    .filter(|t| t.starts_at_coffee_shop())
    .min_by_key(|t| t.duration());
```

Did something bother you?

`Tour::random` function needs a random number generator (RNG). However, RNGs are stateful and are only useful with mutable references. This means, if we create one RNG and share it with all threads, we would have a race condition, and hence, an undefined behavior.

Therefore, in the example above, we create a new random RNG per created tour. This is not how we normally use RNGs; we normally create one and consume the sequence of random numbers it produces.

We know that we cannot work with one RNG due to race conditions, but we could have `T` RNGs where `T` is the number of threads.

With `orx-parallel`, we can ask the parallel iterator to create exactly one RNG per thread and make it available to iterator methods.

```rust
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

struct Tour(Vec<usize>);

impl Tour {
    fn random(rnd: &mut impl Rng, n: usize) -> Self {
        let mut cities: Vec<_> = (0..n).collect();
        cities.shuffle(rnd);
        Self(cities)
    }

    fn starts_at_coffee_shop(&self) -> bool {
        self.0.first() == Some(&7)
    }

    fn duration(&self) -> u64 {
        let links = self.0.iter().zip(self.0.iter().skip(1));
        links
            .map(|(a, b)| (*a as i64 - *b as i64).unsigned_abs())
            .sum::<u64>()
    }
}

let num_tours = 1_000_000;
let num_cities = 10;

// sequential
let mut rnd = rand::rng();
let best_tour = (0..num_tours)
    .map(|_| Tour::random(&mut rnd, num_cities))
    .filter(|t| t.starts_at_coffee_shop())
    .min_by_key(|t| t.duration());

let best_tour = (0..num_tours)
    .par() // ← parallelized
    .use_new(|th_idx| ChaCha8Rng::seed_from_u64(42 * th_idx as u64)) // ← seeded
    .map(|rng, _| Tour::random(rng, num_cities)) // ← a mut rng is now available
    .filter(|_, t| t.starts_at_coffee_shop())
    .min_by_key(|_, t| t.duration());
```

## `use_vec`: Collecting thread local information

> `use_vec` creates one variable per thread, holds them in a vector of length `T`, and can be used after the parallel computation finalizes.


