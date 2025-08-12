use crate::using::{UsingClone, UsingFun};
use crate::{
    ParIterUsing, Params,
    collect_into::ParCollectInto,
    computations::{map_clone, map_copy, map_count, reduce_sum, reduce_unit},
    parameters::{ChunkSize, IterationOrder, NumThreads},
    runner::{DefaultRunner, ParallelRunner},
    special_type_sets::Sum,
};
use orx_concurrent_iter::ConcurrentIter;
use std::cmp::Ordering;

/// Parallel iterator.
pub trait ParIter<R = DefaultRunner>: Sized + Send + Sync
where
    R: ParallelRunner,
{
    /// Element type of the parallel iterator.
    type Item;

    /// Returns a reference to the input concurrent iterator.
    fn con_iter(&self) -> &impl ConcurrentIter;

    /// Parameters of the parallel iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    /// use std::num::NonZero;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// assert_eq!(
    ///     vec.par().params(),
    ///     Params::new(NumThreads::Auto, ChunkSize::Auto, IterationOrder::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(0).chunk_size(0).params(),
    ///     Params::new(NumThreads::Auto, ChunkSize::Auto, IterationOrder::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(1).params(),
    ///     Params::new(
    ///         NumThreads::Max(NonZero::new(1).unwrap()),
    ///         ChunkSize::Auto,
    ///         IterationOrder::Ordered
    ///     )
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(4).chunk_size(64).params(),
    ///     Params::new(
    ///         NumThreads::Max(NonZero::new(4).unwrap()),
    ///         ChunkSize::Exact(NonZero::new(64).unwrap()),
    ///         IterationOrder::Ordered
    ///     )
    /// );
    ///
    /// assert_eq!(
    ///     vec.par()
    ///         .num_threads(8)
    ///         .chunk_size(ChunkSize::Min(NonZero::new(16).unwrap()))
    ///         .iteration_order(IterationOrder::Arbitrary)
    ///         .params(),
    ///     Params::new(
    ///         NumThreads::Max(NonZero::new(8).unwrap()),
    ///         ChunkSize::Min(NonZero::new(16).unwrap()),
    ///         IterationOrder::Arbitrary
    ///     )
    /// );
    /// ```
    fn params(&self) -> Params;

    // params transformations

    /// Sets the number of threads to be used in the parallel execution.
    /// Integers can be used as the argument with the following mapping:
    ///
    /// * `0` -> `NumThreads::Auto`
    /// * `1` -> `NumThreads::sequential()`
    /// * `n > 0` -> `NumThreads::Max(n)`
    ///
    /// See [`NumThreads`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    /// use std::num::NonZero;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// // all available threads can be used
    ///
    /// assert_eq!(
    ///     vec.par().params(),
    ///     Params::new(NumThreads::Auto, ChunkSize::Auto, IterationOrder::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(0).chunk_size(0).params(),
    ///     Params::new(NumThreads::Auto, ChunkSize::Auto, IterationOrder::Ordered)
    /// );
    ///
    /// // computation will be executed sequentially on the main thread, no parallelization
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(1).params(),
    ///     Params::new(
    ///         NumThreads::Max(NonZero::new(1).unwrap()),
    ///         ChunkSize::Auto,
    ///         IterationOrder::Ordered
    ///     )
    /// );
    ///
    /// // maximum 4 threads can be used
    /// assert_eq!(
    ///     vec.par().num_threads(4).chunk_size(64).params(),
    ///     Params::new(
    ///         NumThreads::Max(NonZero::new(4).unwrap()),
    ///         ChunkSize::Exact(NonZero::new(64).unwrap()),
    ///         IterationOrder::Ordered
    ///     )
    /// );
    ///
    /// // maximum 8 threads can be used
    /// assert_eq!(
    ///     vec.par()
    ///         .num_threads(8)
    ///         .chunk_size(ChunkSize::Min(NonZero::new(16).unwrap()))
    ///         .iteration_order(IterationOrder::Arbitrary)
    ///         .params(),
    ///     Params::new(
    ///         NumThreads::Max(NonZero::new(8).unwrap()),
    ///         ChunkSize::Min(NonZero::new(16).unwrap()),
    ///         IterationOrder::Arbitrary
    ///     )
    /// );
    /// ```
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Sets the number of elements to be pulled from the concurrent iterator during the
    /// parallel execution. When integers are used as argument, the following mapping applies:
    ///
    /// * `0` -> `ChunkSize::Auto`
    /// * `n > 0` -> `ChunkSize::Exact(n)`
    ///
    /// Please use the default enum constructor for creating `ChunkSize::Min` variant.
    ///
    /// See [`ChunkSize`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    /// use std::num::NonZero;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// // chunk sizes will be dynamically decided by the parallel runner
    ///
    /// assert_eq!(
    ///     vec.par().params(),
    ///     Params::new(NumThreads::Auto, ChunkSize::Auto, IterationOrder::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(0).chunk_size(0).params(),
    ///     Params::new(NumThreads::Auto, ChunkSize::Auto, IterationOrder::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(1).params(),
    ///     Params::new(
    ///         NumThreads::Max(NonZero::new(1).unwrap()),
    ///         ChunkSize::Auto,
    ///         IterationOrder::Ordered
    ///     )
    /// );
    ///
    /// // chunk size will always be 64, parallel runner cannot change
    ///
    /// assert_eq!(
    ///     vec.par().num_threads(4).chunk_size(64).params(),
    ///     Params::new(
    ///         NumThreads::Max(NonZero::new(4).unwrap()),
    ///         ChunkSize::Exact(NonZero::new(64).unwrap()),
    ///         IterationOrder::Ordered
    ///     )
    /// );
    ///
    /// // minimum chunk size will be 16, but can be dynamically increased by the parallel runner
    ///
    /// assert_eq!(
    ///     vec.par()
    ///         .num_threads(8)
    ///         .chunk_size(ChunkSize::Min(NonZero::new(16).unwrap()))
    ///         .iteration_order(IterationOrder::Arbitrary)
    ///         .params(),
    ///     Params::new(
    ///         NumThreads::Max(NonZero::new(8).unwrap()),
    ///         ChunkSize::Min(NonZero::new(16).unwrap()),
    ///         IterationOrder::Arbitrary
    ///     )
    /// );
    /// ```
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Sets the iteration order of the parallel computation.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let vec = vec![1, 2, 3, 4];
    ///
    /// // results are collected in order consistent to the input order,
    /// // or find returns the first element satisfying the predicate
    ///
    /// assert_eq!(
    ///     vec.par().params(),
    ///     Params::new(NumThreads::Auto, ChunkSize::Auto, IterationOrder::Ordered)
    /// );
    ///
    /// assert_eq!(
    ///     vec.par().iteration_order(IterationOrder::Ordered).params(),
    ///     Params::new(NumThreads::Auto, ChunkSize::Auto, IterationOrder::Ordered)
    /// );
    ///
    /// // results might be collected in arbitrary order
    /// // or find returns the any of the elements satisfying the predicate
    ///
    /// assert_eq!(
    ///     vec.par().iteration_order(IterationOrder::Arbitrary).params(),
    ///     Params::new(NumThreads::Auto, ChunkSize::Auto, IterationOrder::Arbitrary)
    /// );
    /// ```
    fn iteration_order(self, collect: IterationOrder) -> Self;

    /// Rather than the [`DefaultRunner`], uses the parallel runner `Q` which implements [`ParallelRunner`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use orx_parallel::*;
    ///
    /// let inputs = vec![1, 2, 3, 4];
    ///
    /// // uses the default runner
    /// let sum = inputs.par().sum();
    ///
    /// // uses the custom parallel runner MyParallelRunner: ParallelRunner
    /// let sum = inputs.par().with_runner::<MyParallelRunner>().sum();
    /// ```
    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item>;

    // using transformations

    /// Converts the [`ParIter`] into [`ParIterUsing`] which will have access to a mutable reference of the
    /// used variable throughout the computation.
    ///
    /// Note that each used thread will obtain exactly one instance of the variable.
    ///
    /// The signature of the `using` closure is `(thread_idx: usize) -> U` which will create an instance of
    /// `U` with respect to the `thread_idx`. The `thread_idx` is the order of the spawned thread; i.e.,
    /// if the parallel computation uses 8 threads, the thread indices will be 0, 1, ..., 7.
    ///
    /// Details of the **using transformation** can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    ///
    /// # Examples
    ///
    /// ## Example 1: Channels
    ///
    /// The following example is taken from rayon's `for_each_with` documentation and converted to using transformation:
    ///
    /// ```ignore
    /// use orx_parallel::*;
    /// use std::sync::mpsc::channel;
    ///
    /// let (sender, receiver) = channel();
    ///
    /// (0..5)
    ///     .into_par()
    ///     .using(|_thread_idx| sender.clone())
    ///     .for_each(|s, x| s.send(x).unwrap());
    ///
    /// let mut res: Vec<_> = receiver.iter().collect();
    ///
    /// res.sort();
    ///
    /// assert_eq!(&res[..], &[0, 1, 2, 3, 4])
    /// ```
    ///
    /// ## Example 2: Random Number Generator
    ///
    /// Random number generator is one of the common use cases that is important for a certain class of algorithms.
    ///
    /// The following example demonstrates how to safely generate random numbers through mutable references within
    /// a parallel computation.
    ///
    /// Notice the differences between sequential and parallel computation.
    /// * In sequential computation, a mutable reference to `rng` is captured, while in parallel computation, we
    ///   explicitly define that we will be `using` a random number generator.
    /// * Parallel iterator does not mutable capture any variable from the scope; however, using transformation
    ///   converts the `ParIter` into `ParIterUsing` which allows mutable access within all iterator methods.
    ///
    /// ```
    /// use orx_parallel::*;
    /// use rand::{Rng, SeedableRng};
    /// use rand_chacha::ChaCha20Rng;
    ///
    /// fn random_walk(rng: &mut impl Rng, position: i64, num_steps: usize) -> i64 {
    ///     (0..num_steps).fold(position, |p, _| random_step(rng, p))
    /// }
    ///
    /// fn random_step(rng: &mut impl Rng, position: i64) -> i64 {
    ///     match rng.random_bool(0.5) {
    ///         true => position + 1,  // to right
    ///         false => position - 1, // to left
    ///     }
    /// }
    ///
    /// fn input_positions() -> Vec<i64> {
    ///     (-100..=100).collect()
    /// }
    ///
    /// fn sequential() {
    ///     let positions = input_positions();
    ///
    ///     let mut rng = ChaCha20Rng::seed_from_u64(42);
    ///     let final_positions: Vec<_> = positions
    ///         .iter()
    ///         .copied()
    ///         .map(|position| random_walk(&mut rng, position, 10))
    ///         .collect();
    ///     let sum_final_positions = final_positions.iter().sum::<i64>();
    /// }
    ///
    /// fn parallel() {
    ///     let positions = input_positions();
    ///
    ///     let final_positions: Vec<_> = positions
    ///         .par()
    ///         .copied()
    ///         .using(|t_idx| ChaCha20Rng::seed_from_u64(42 * t_idx as u64))
    ///         .map(|rng, position| random_walk(rng, position, 10))
    ///         .collect();
    ///     let sum_final_positions = final_positions.iter().sum::<i64>();
    /// }
    ///
    /// sequential();
    /// parallel();
    /// ```
    ///
    /// ## Example 3: Metrics Collection
    ///
    /// The following example demonstrates how to collect metrics about a parallel computation with `using` transformation and
    /// some `unsafe` help with interior mutability.
    ///
    /// ```
    /// use orx_parallel::*;
    /// use std::cell::UnsafeCell;
    ///
    /// const N: u64 = 1_000;
    /// const MAX_NUM_THREADS: usize = 4;
    ///
    /// // just some work
    /// fn fibonacci(n: u64) -> u64 {
    ///     let mut a = 0;
    ///     let mut b = 1;
    ///     for _ in 0..n {
    ///         let c = a + b;
    ///         a = b;
    ///         b = c;
    ///     }
    ///     a
    /// }
    ///
    /// #[derive(Default, Debug)]
    /// struct ThreadMetrics {
    ///     thread_idx: usize,
    ///     num_items_handled: usize,
    ///     handled_42: bool,
    ///     num_filtered_out: usize,
    /// }
    ///
    /// struct ThreadMetricsWriter<'a> {
    ///     metrics_ref: &'a mut ThreadMetrics,
    /// }
    ///
    /// struct ComputationMetrics {
    ///     thread_metrics: UnsafeCell<[ThreadMetrics; MAX_NUM_THREADS]>,
    /// }
    /// impl ComputationMetrics {
    ///     fn new() -> Self {
    ///         let mut thread_metrics: [ThreadMetrics; MAX_NUM_THREADS] = Default::default();
    ///         for i in 0..MAX_NUM_THREADS {
    ///             thread_metrics[i].thread_idx = i;
    ///         }
    ///         Self {
    ///             thread_metrics: UnsafeCell::new(thread_metrics),
    ///         }
    ///     }
    /// }
    ///
    /// impl ComputationMetrics {
    ///     unsafe fn create_for_thread<'a>(&mut self, thread_idx: usize) -> ThreadMetricsWriter<'a> {
    ///         // SAFETY: here we create a mutable variable to the thread_idx-th metrics
    ///         // * If we call this method multiple times with the same index,
    ///         //   we create multiple mutable references to the same ThreadMetrics,
    ///         //   which would lead to a race condition.
    ///         // * We must make sure that `create_for_thread` is called only once per thread.
    ///         // * If we use `create_for_thread` within the `using` call to create mutable values
    ///         //   used by the threads, we are certain that the parallel computation
    ///         //   will only call this method once per thread; hence, it will not
    ///         //   cause the race condition.
    ///         // * On the other hand, we must ensure that we do not call this method
    ///         //   externally.
    ///         let array = unsafe { &mut *self.thread_metrics.get() };
    ///         ThreadMetricsWriter {
    ///             metrics_ref: &mut array[thread_idx],
    ///         }
    ///     }
    /// }
    ///
    /// let mut metrics = ComputationMetrics::new();
    ///
    /// let input: Vec<u64> = (0..N).collect();
    ///
    /// let sum = input
    ///     .par()
    ///     // SAFETY: we do not call `create_for_thread` externally;
    ///     // it is safe if it is called only by the parallel computation.
    ///     .using(|t| unsafe { metrics.create_for_thread(t) })
    ///     .map(|m: &mut ThreadMetricsWriter<'_>, i| {
    ///         // collect some useful metrics
    ///         m.metrics_ref.num_items_handled += 1;
    ///         m.metrics_ref.handled_42 |= *i == 42;
    ///
    ///         // actual work
    ///         fibonacci((*i % 20) + 1) % 100
    ///     })
    ///     .filter(|m, i| {
    ///         let is_even = i % 2 == 0;
    ///
    ///         if !is_even {
    ///             m.metrics_ref.num_filtered_out += 1;
    ///         }
    ///
    ///         is_even
    ///     })
    ///     .num_threads(MAX_NUM_THREADS)
    ///     .sum();
    ///
    /// let total_by_metrics: usize = metrics
    ///     .thread_metrics
    ///     .get_mut()
    ///     .iter()
    ///     .map(|x| x.num_items_handled)
    ///     .sum();
    ///
    /// assert_eq!(N as usize, total_by_metrics);
    /// ```
    ///
    fn using<U, F>(
        self,
        using: F,
    ) -> impl ParIterUsing<UsingFun<F, U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Send,
        F: FnMut(usize) -> U;

    /// Converts the [`ParIter`] into [`ParIterUsing`] which will have access to a mutable reference of the
    /// used variable throughout the computation.
    ///
    /// Note that each used thread will obtain exactly one instance of the variable.
    ///
    /// Each used thread receives a clone of the provided `value`.
    /// Note that, `using_clone(value)` can be considered as a shorthand for `using(|_thread_idx| value.clone())`.
    ///
    /// Please see [`using`] for examples.
    ///
    /// Details of the **using transformation** can be found here: [`using.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).
    ///
    /// [`using`]: crate::ParIter::using
    fn using_clone<U>(
        self,
        value: U,
    ) -> impl ParIterUsing<UsingClone<U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Clone + Send;

    // computation transformations

    /// Takes a closure `map` and creates a parallel iterator which calls that closure on each element.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = [1, 2, 3];
    ///
    /// let iter = a.into_par().map(|x| 2 * x);
    ///
    /// let b: Vec<_> = iter.collect();
    /// assert_eq!(b, &[2, 4, 6]);
    /// ```
    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Map: Fn(Self::Item) -> Out + Sync + Clone;

    /// Creates an iterator which uses a closure `filter` to determine if an element should be yielded.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = [1, 2, 3];
    ///
    /// let iter = a.into_par().filter(|x| *x % 2 == 1).copied();
    ///
    /// let b: Vec<_> = iter.collect();
    /// assert_eq!(b, &[1, 3]);
    /// ```
    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Sync + Clone;

    /// Creates an iterator that works like map, but flattens nested structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let words = ["alpha", "beta", "gamma"];
    ///
    /// // chars() returns an iterator
    /// let all_chars: Vec<_> = words.into_par().flat_map(|s| s.chars()).collect();
    ///
    /// let merged: String = all_chars.iter().collect();
    /// assert_eq!(merged, "alphabetagamma");
    /// ```
    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(Self::Item) -> IOut + Sync + Clone;

    /// Creates an iterator that both filters and maps.
    ///
    /// The returned iterator yields only the values for which the supplied closure `filter_map` returns `Some(value)`.
    ///
    /// `filter_map` can be used to make chains of `filter` and `map` more concise.
    /// The example below shows how a `map().filter().map()` can be shortened to a single call to `filter_map`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = ["1", "two", "NaN", "four", "5"];
    ///
    /// let numbers: Vec<_> = a
    ///     .into_par()
    ///     .filter_map(|s| s.parse::<usize>().ok())
    ///     .collect();
    ///
    /// assert_eq!(numbers, [1, 5]);
    /// ```
    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync + Clone;

    fn whilst<Whilst>(self, whilst: Whilst) -> impl ParIter<R, Item = Self::Item>
    where
        Whilst: Fn(&Self::Item) -> bool + Sync + Clone;

    /// Does something with each element of an iterator, passing the value on.
    ///
    /// When using iterators, you’ll often chain several of them together.
    /// While working on such code, you might want to check out what’s happening at various parts in the pipeline.
    /// To do that, insert a call to `inspect()`.
    ///
    /// It’s more common for `inspect()` to be used as a debugging tool than to exist in your final code,
    /// but applications may find it useful in certain situations when errors need to be logged before being discarded.
    ///
    /// It is often convenient to use thread-safe collections such as [`ConcurrentBag`] and
    /// [`ConcurrentVec`](https://crates.io/crates/orx-concurrent-vec) to
    /// collect some intermediate values during parallel execution for further inspection.
    /// The following example demonstrates such a use case.
    ///
    /// [`ConcurrentBag`]: orx_concurrent_bag::ConcurrentBag
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let a = vec![1, 4, 2, 3];
    ///
    /// // let's add some inspect() calls to investigate what's happening
    /// // - log some events
    /// // - use a concurrent bag to collect and investigate numbers contributing to the sum
    /// let bag = ConcurrentBag::new();
    ///
    /// let sum = a
    ///     .par()
    ///     .copied()
    ///     .inspect(|x| println!("about to filter: {x}"))
    ///     .filter(|x| x % 2 == 0)
    ///     .inspect(|x| {
    ///         bag.push(*x);
    ///         println!("made it through filter: {x}");
    ///     })
    ///     .sum();
    /// println!("{sum}");
    ///
    /// let mut values_made_through = bag.into_inner();
    /// values_made_through.sort();
    /// assert_eq!(values_made_through, [2, 4]);
    /// ```
    ///
    /// This will print:
    ///
    /// ```console
    /// about to filter: 1
    /// about to filter: 4
    /// made it through filter: 4
    /// about to filter: 2
    /// made it through filter: 2
    /// about to filter: 3
    /// 6
    /// ```
    fn inspect<Operation>(self, operation: Operation) -> impl ParIter<R, Item = Self::Item>
    where
        Operation: Fn(&Self::Item) + Sync + Clone,
    {
        let map = move |x| {
            operation(&x);
            x
        };
        self.map(map)
    }

    // computation transformations - derived from whilst

    fn map_while<Out, MapWhile>(self, map_while: MapWhile) -> impl ParIter<R, Item = Out>
    where
        MapWhile: Fn(Self::Item) -> Option<Out> + Sync + Clone,
    {
        self.map(map_while).whilst(|x| x.is_some()).map(|x| {
            // SAFETY: since x passed the whilst(is-some) check, unwrap_unchecked
            unsafe { x.unwrap_unchecked() }
        })
    }

    // special item transformations

    /// Creates an iterator which copies all of its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    ///
    /// let v_copied: Vec<_> = a.par().copied().collect();
    ///
    /// // copied is the same as .map(|&x| x)
    /// let v_map: Vec<_> = a.par().map(|&x| x).collect();
    ///
    /// assert_eq!(v_copied, vec![1, 2, 3]);
    /// assert_eq!(v_map, vec![1, 2, 3]);
    /// ```
    fn copied<'a, T>(self) -> impl ParIter<R, Item = T>
    where
        T: 'a + Copy,
        Self: ParIter<R, Item = &'a T>,
    {
        self.map(map_copy)
    }

    /// Creates an iterator which clones all of its elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<_> = [1, 2, 3].map(|x| x.to_string()).into_iter().collect();
    ///
    /// let v_cloned: Vec<_> = a.par().cloned().collect();
    ///
    /// // cloned is the same as .map(|x| x.clone())
    /// let v_map: Vec<_> = a.par().map(|x| x.clone()).collect();
    ///
    /// assert_eq!(
    ///     v_cloned,
    ///     vec![String::from("1"), String::from("2"), String::from("3")]
    /// );
    /// assert_eq!(
    ///     v_map,
    ///     vec![String::from("1"), String::from("2"), String::from("3")]
    /// );
    /// ```
    fn cloned<'a, T>(self) -> impl ParIter<R, Item = T>
    where
        T: 'a + Clone,
        Self: ParIter<R, Item = &'a T>,
    {
        self.map(map_clone)
    }

    /// Creates an iterator that flattens nested structure.
    ///
    /// This is useful when you have an iterator of iterators or an iterator of things that can be
    /// turned into iterators and you want to remove one level of indirection.
    ///
    /// # Examples
    ///
    /// Basic usage.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let data = vec![vec![1, 2, 3, 4], vec![5, 6]];
    /// let flattened = data.into_par().flatten().collect::<Vec<u8>>();
    /// assert_eq!(flattened, &[1, 2, 3, 4, 5, 6]);
    /// ```
    ///
    /// Mapping and then flattening:
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let words = vec!["alpha", "beta", "gamma"];
    ///
    /// // chars() returns an iterator
    /// let all_characters: Vec<_> = words.par().map(|s| s.chars()).flatten().collect();
    /// let merged: String = all_characters.into_iter().collect();
    /// assert_eq!(merged, "alphabetagamma");
    /// ```
    ///
    /// But actually, you can write this in terms of `flat_map`,
    /// which is preferable in this case since it conveys intent more clearly:
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let words = vec!["alpha", "beta", "gamma"];
    ///
    /// // chars() returns an iterator
    /// let all_characters: Vec<_> = words.par().flat_map(|s| s.chars()).collect();
    /// let merged: String = all_characters.into_iter().collect();
    /// assert_eq!(merged, "alphabetagamma");
    /// ```
    fn flatten(self) -> impl ParIter<R, Item = <Self::Item as IntoIterator>::Item>
    where
        Self::Item: IntoIterator,
    {
        let map = |e: Self::Item| e.into_iter();
        self.flat_map(map)
    }

    // collect

    /// Collects all the items from an iterator into a collection.
    ///
    /// This is useful when you already have a collection and want to add the iterator items to it.
    ///
    /// The collection is passed in as owned value, and returned back with the additional elements.
    ///
    /// All collections implementing [`ParCollectInto`] can be used to collect into.
    ///
    /// [`ParCollectInto`]: crate::ParCollectInto
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    ///
    /// let vec: Vec<i32> = vec![0, 1];
    /// let vec = a.par().map(|&x| x * 2).collect_into(vec);
    /// let vec = a.par().map(|&x| x * 10).collect_into(vec);
    ///
    /// assert_eq!(vec, vec![0, 1, 2, 4, 6, 10, 20, 30]);
    /// ```
    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>;

    /// Transforms an iterator into a collection.
    ///
    /// Similar to [`Iterator::collect`], the type annotation on the left-hand-side determines
    /// the type of the result collection; or turbofish annotation can be used.
    ///
    /// All collections implementing [`ParCollectInto`] can be used to collect into.
    ///
    /// [`ParCollectInto`]: crate::ParCollectInto
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    ///
    /// let doubled: Vec<i32> = a.par().map(|&x| x * 2).collect();
    ///
    /// assert_eq!(vec![2, 4, 6], doubled);
    /// ```
    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let output = C::empty(self.con_iter().try_get_len());
        self.collect_into(output)
    }

    // reduce

    /// Reduces the elements to a single one, by repeatedly applying a reducing operation.
    ///
    /// If the iterator is empty, returns `None`; otherwise, returns the result of the reduction.
    ///
    /// The `reduce` function is a closure with two arguments: an ‘accumulator’, and an element.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let inputs = 1..10;
    /// let reduced: usize = inputs.par().reduce(|acc, e| acc + e).unwrap_or(0);
    /// assert_eq!(reduced, 45);
    /// ```
    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync;

    /// Tests if every element of the iterator matches a predicate.
    ///
    /// `all` takes a `predicate` that returns true or false.
    /// It applies this closure to each element of the iterator,
    /// and if they all return true, then so does `all`.
    /// If any of them returns false, it returns false.
    ///
    /// `all` is short-circuiting; in other words, it will stop processing as soon as it finds a false,
    /// given that no matter what else happens, the result will also be false.
    ///
    /// An empty iterator returns true.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut a = vec![1, 2, 3];
    /// assert!(a.par().all(|x| **x > 0));
    /// assert!(!a.par().all(|x| **x > 2));
    ///
    /// a.clear();
    /// assert!(a.par().all(|x| **x > 2)); // empty iterator
    /// ```
    fn all<Predicate>(self, predicate: Predicate) -> bool
    where
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        let violates = |x: &Self::Item| !predicate(x);
        self.find(violates).is_none()
    }

    /// Tests if any element of the iterator matches a predicate.
    ///
    /// `any` takes a `predicate` that returns true or false.
    /// It applies this closure to each element of the iterator,
    /// and if any of the elements returns true, then so does `any`.
    /// If all of them return false, it returns false.
    ///
    /// `any` is short-circuiting; in other words, it will stop processing as soon as it finds a true,
    /// given that no matter what else happens, the result will also be true.
    ///
    /// An empty iterator returns false.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut a = vec![1, 2, 3];
    /// assert!(a.par().any(|x| **x > 0));
    /// assert!(!a.par().any(|x| **x > 5));
    ///
    /// a.clear();
    /// assert!(!a.par().any(|x| **x > 0)); // empty iterator
    /// ```
    fn any<Predicate>(self, predicate: Predicate) -> bool
    where
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        self.find(predicate).is_some()
    }

    /// Consumes the iterator, counting the number of iterations and returning it.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    /// assert_eq!(a.par().filter(|x| **x >= 2).count(), 2);
    /// ```
    fn count(self) -> usize {
        self.map(map_count).reduce(reduce_sum).unwrap_or(0)
    }

    /// Calls a closure on each element of an iterator.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use orx_parallel::*;
    /// use std::sync::mpsc::channel;
    ///
    /// let (tx, rx) = channel();
    /// (0..5)
    ///     .par()
    ///     .map(|x| x * 2 + 1)
    ///     .for_each(move |x| tx.send(x).unwrap());
    ///
    /// let mut v: Vec<_> = rx.iter().collect();
    /// v.sort(); // order can be mixed, since messages will be sent in parallel
    /// assert_eq!(v, vec![1, 3, 5, 7, 9]);
    /// ```
    ///
    /// Note that since parallel iterators cannot be used within the `for` loop as regular iterators,
    /// `for_each` provides a way to perform arbitrary for loops on parallel iterators.
    /// In the following example, we log every element that satisfies a predicate in parallel.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// (0..5)
    ///     .par()
    ///     .flat_map(|x| x * 100..x * 110)
    ///     .filter(|&x| x % 3 == 0)
    ///     .for_each(|x| println!("{x}"));
    /// ```
    fn for_each<Operation>(self, operation: Operation)
    where
        Operation: Fn(Self::Item) + Sync,
    {
        let map = |x| operation(x);
        let _ = self.map(map).reduce(reduce_unit);
    }

    /// Returns the maximum element of an iterator.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    /// let b: Vec<u32> = Vec::new();
    ///
    /// assert_eq!(a.par().max(), Some(&3));
    /// assert_eq!(b.par().max(), None);
    /// ```
    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    /// Returns the element that gives the maximum value with respect to the specified `compare` function.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![-3_i32, 0, 1, 5, -10];
    /// assert_eq!(*a.par().max_by(|x, y| x.cmp(y)).unwrap(), 5);
    /// ```
    fn max_by<Compare>(self, compare: Compare) -> Option<Self::Item>
    where
        Self::Item: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the maximum value from the specified function.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![-3_i32, 0, 1, 5, -10];
    /// assert_eq!(*a.par().max_by_key(|x| x.abs()).unwrap(), -10);
    /// ```
    fn max_by_key<Key, GetKey>(self, key: GetKey) -> Option<Self::Item>
    where
        Self::Item: Send,
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |x, y| match key(&x).cmp(&key(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    /// Returns the minimum element of an iterator.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    /// let b: Vec<u32> = Vec::new();
    ///
    /// assert_eq!(a.par().min(), Some(&1));
    /// assert_eq!(b.par().min(), None);
    /// ```
    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    /// Returns the element that gives the minimum value with respect to the specified `compare` function.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![-3_i32, 0, 1, 5, -10];
    /// assert_eq!(*a.par().min_by(|x, y| x.cmp(y)).unwrap(), -10);
    /// ```
    fn min_by<Compare>(self, compare: Compare) -> Option<Self::Item>
    where
        Self::Item: Send,
        Compare: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match compare(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Returns the element that gives the minimum value from the specified function.
    ///
    /// If the iterator is empty, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![-3_i32, 0, 1, 5, -10];
    /// assert_eq!(*a.par().min_by_key(|x| x.abs()).unwrap(), 0);
    /// ```
    fn min_by_key<Key, GetKey>(self, get_key: GetKey) -> Option<Self::Item>
    where
        Self::Item: Send,
        Key: Ord,
        GetKey: Fn(&Self::Item) -> Key + Sync,
    {
        let reduce = |x, y| match get_key(&x).cmp(&get_key(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    /// Sums the elements of an iterator.
    ///
    /// Takes each element, adds them together, and returns the result.
    ///
    /// An empty iterator returns the additive identity (“zero”) of the type, which is 0 for integers and -0.0 for floats.
    ///
    /// `sum` can be used to sum any type implementing [`Sum<Out>`].
    ///
    /// [`Sum<Out>`]: crate::Sum
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = vec![1, 2, 3];
    /// let sum: i32 = a.par().sum();
    ///
    /// assert_eq!(sum, 6);
    /// ```
    fn sum<Out>(self) -> Out
    where
        Self::Item: Sum<Out>,
        Out: Send,
    {
        self.map(Self::Item::map)
            .reduce(Self::Item::reduce)
            .unwrap_or(Self::Item::zero())
    }

    // early exit

    /// Returns the first (or any) element of the iterator; returns None if it is empty.
    ///
    /// * first element is returned if default iteration order `IterationOrder::Ordered` is used,
    /// * any element is returned if `IterationOrder::Arbitrary` is set.
    ///
    /// # Examples
    ///
    /// The following example demonstrates the usage of first with default `Ordered` iteration.
    /// This guarantees that the first element with respect to position in the input sequence
    /// is returned.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a: Vec<usize> = vec![];
    /// assert_eq!(a.par().copied().first(), None);
    ///
    /// let a = vec![1, 2, 3];
    /// assert_eq!(a.par().copied().first(), Some(1));
    ///
    /// let a = 1..10_000;
    /// assert_eq!(a.par().filter(|x| x % 3421 == 0).first(), Some(3421));
    /// assert_eq!(a.par().filter(|x| x % 12345 == 0).first(), None);
    ///
    /// // or equivalently,
    /// assert_eq!(a.par().find(|x| x % 3421 == 0), Some(3421));
    /// ```
    ///
    /// When the order is set to `Arbitrary`, `first` might return any of the elements,
    /// whichever is visited first depending on the parallel execution.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = 1..10_000;
    ///
    /// // might return either of 3421 or 2*3421
    /// let any = a.par().iteration_order(IterationOrder::Arbitrary).filter(|x| x % 3421 == 0).first().unwrap();
    /// assert!([3421, 2 * 3421].contains(&any));
    ///
    /// // or equivalently,
    /// let any = a.par().iteration_order(IterationOrder::Arbitrary).find(|x| x % 3421 == 0).unwrap();
    /// assert!([3421, 2 * 3421].contains(&any));
    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send;

    /// Searches for an element of an iterator that satisfies a `predicate`.
    ///
    /// Depending on the set iteration order of the parallel iterator, returns
    ///
    /// * first element satisfying the `predicate` if default iteration order `IterationOrder::Ordered` is used,
    /// * any element satisfying the `predicate` if `IterationOrder::Arbitrary` is set.
    ///
    /// `find` takes a closure that returns true or false.
    /// It applies this closure to each element of the iterator,
    /// and returns `Some(x)` where `x` is the first element that returns true.
    /// If they all return false, it returns None.
    ///
    /// `find` is short-circuiting; in other words, it will stop processing as soon as the closure returns true.
    ///
    /// `par_iter.find(predicate)` can also be considered as a shorthand for `par_iter.filter(predicate).first()`.
    ///
    /// # Examples
    ///
    /// The following example demonstrates the usage of first with default `Ordered` iteration.
    /// This guarantees that the first element with respect to position in the input sequence
    /// is returned.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = 1..10_000;
    /// assert_eq!(a.par().find(|x| x % 12345 == 0), None);
    /// assert_eq!(a.par().find(|x| x % 3421 == 0), Some(3421));
    /// ```
    ///
    /// When the order is set to `Arbitrary`, `find` might return any of the elements satisfying the predicate,
    /// whichever is found first depending on the parallel execution.
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let a = 1..10_000;
    ///
    /// // might return either of 3421 or 2*3421
    /// let any = a.par().iteration_order(IterationOrder::Arbitrary).find(|x| x % 3421 == 0).unwrap();
    /// assert!([3421, 2 * 3421].contains(&any));
    /// ```
    fn find<Predicate>(self, predicate: Predicate) -> Option<Self::Item>
    where
        Self::Item: Send,
        Predicate: Fn(&Self::Item) -> bool + Sync,
    {
        self.filter(&predicate).first()
    }
}
