use crate::{
    par::collect_into::par_collect_into::ParCollectInto, ChunkSize, NumThreads, Params, Reduce,
};
use orx_split_vec::SplitVec;

/// An iterator used to define a computation that can be executed in parallel.
pub trait ParIter: Reduce<Self::Item> {
    /// Type of the items that the iterator yields.
    type Item: Send + Sync + Default;

    /// Parameters of the parallel computation which can be set by `num_threads` and `chunk_size` methods.
    fn params(&self) -> Params;

    /// Transforms the parallel computation with a new one with the given `num_threads`.
    ///
    /// See [`crate::NumThreads`] for details.
    ///
    /// `num_threads` represents the degree of parallelization. It is possible to define an upper bound on the number of threads to be used for the parallel computation.
    /// When set to **1**, the computation will be executed sequentially without any overhead.
    /// In this sense, parallel iterators defined in this crate are a union of sequential and parallel execution.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use std::num::NonZeroUsize;
    ///
    /// let expected = (0..(1 << 20)).sum();
    ///
    /// // unset/default -> NumThreads::Auto
    /// let sum = (0..(1 << 20)).into_par().sum();
    /// assert_eq!(sum, expected);
    ///
    /// // A: NumThreads::Auto
    /// let sum = (0..(1 << 20)).into_par().num_threads(0).sum();
    /// assert_eq!(sum, expected);
    ///
    /// let sum = (0..(1 << 20)).into_par().num_threads(NumThreads::Auto).sum();
    /// assert_eq!(sum, expected);
    ///
    /// // B: with a limit on the number of threads
    /// let sum = (0..(1 << 20)).into_par().num_threads(4).sum();
    /// assert_eq!(sum, expected);
    ///
    /// let sum = (0..(1 << 20)).into_par().num_threads(NumThreads::Max(NonZeroUsize::new(4).unwrap())).sum();
    /// assert_eq!(sum, expected);
    ///
    /// // C: sequential execution
    /// let sum = (0..(1 << 20)).into_par().num_threads(1).sum();
    /// assert_eq!(sum, expected);
    ///
    /// let sum = (0..(1 << 20)).into_par().num_threads(NumThreads::sequential()).sum();
    /// assert_eq!(sum, expected);
    /// ```
    ///
    /// # Rules of Thumb / Guidelines
    ///
    /// It is recommended to set this parameter to its default value, `NumThreads::Auto`.
    /// This setting assumes that it can use all available threads; however, the computation will spawn new threads only when required.
    /// In other words, when we can dynamically decide that the task is not large enough to justify spawning a new thread, the parallel execution will avoid it.
    ///
    /// A special case is `NumThreads::Max(NonZeroUsize::new(1).unwrap())`, or equivalently `NumThreads::sequential()`.
    /// This will lead to a sequential execution of the defined computation on the main thread.
    /// Both in terms of used resources and computation time, this mode is not similar but **identical** to a sequential execution using the regular sequential `Iterator`s.
    ///
    /// Lastly, `NumThreads::Max(t)` where `t >= 2` can be used in the following scenarios:
    /// * We have a strict limit on the resources that we can use for this computation, even if the hardware has more resources.
    /// Parallel execution will ensure that `t` will never be exceeded.
    /// * We have a computation which is extremely time-critical and our benchmarks show that `t` outperforms the `NumThreads::Auto` on the corresponding system.
    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    /// Transforms the parallel computation with a new one with the given `chunk_size`.
    ///
    /// See [`crate::ChunkSize`] for details.
    ///
    /// `chunk_size` represents the batch size of elements each thread will pull from the main iterator once it becomes idle again.
    /// It is possible to define a minimum or exact chunk size.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use std::num::NonZeroUsize;
    ///
    /// let expected = (0..(1 << 20)).sum();
    ///
    /// // unset/default -> ChunkSize::Auto
    /// let sum = (0..(1 << 20)).into_par().sum();
    /// assert_eq!(sum, expected);
    ///
    /// // A: ChunkSize::Auto
    /// let sum = (0..(1 << 20)).into_par().chunk_size(0).sum();
    /// assert_eq!(sum, expected);
    ///
    /// let sum = (0..(1 << 20)).into_par().chunk_size(ChunkSize::Auto).sum();
    /// assert_eq!(sum, expected);
    ///
    /// // B: with lower bound on the chunk size, execution may increase chunk size whenever it improves performance
    /// let sum = (0..(1 << 20)).into_par().chunk_size(1024).sum();
    /// assert_eq!(sum, expected);
    ///
    /// let sum = (0..(1 << 20)).into_par().chunk_size(ChunkSize::Min(NonZeroUsize::new(1024).unwrap())).sum();
    /// assert_eq!(sum, expected);
    ///
    /// // C: with an exact chunk size
    /// let sum = (0..(1 << 20)).into_par().chunk_size(ChunkSize::Exact(NonZeroUsize::new(1024).unwrap())).sum();
    /// assert_eq!(sum, expected);
    /// ```
    ///
    /// # Rules of Thumb / Guidelines
    ///
    /// The objective of this parameter is to balance the overhead of parallelization and cost of heterogeneity of tasks.
    ///
    /// In order to illustrate, assume that there exist 8 elements to process, or 8 jobs to execute, and we will use 2 threads for this computation.
    /// Two extreme strategies can be defined as follows.
    ///
    /// * **Perfect Sharing of Tasks**
    ///   * Setting chunk size to 4 provides a perfect division of tasks in terms of quantity.
    /// Each thread will retrieve 4 elements at once in one pull and process them.
    /// This *one pull* per thread can be considered as the parallelization overhead and this is the best/minimum we can achieve.
    ///   * Drawback of this approach, on the other hand, is observed when the execution time of each job is significantly different; i.e., when we have heterogeneous tasks.
    ///   * Assume, for instance, that the first element requires 7 units of time while all remaining elements require 1 unit of time.
    ///   * Roughly, the parallel execution with a chunk size of 4 would complete in 10 units of time, which is the execution time of the first thread (7 + 3*1).
    ///   * The second thread will complete its 4 tasks in 4 units of time and will remain idle for 6 units of time.
    /// * **Perfect Handling of Heterogeneity**
    ///   * Setting chunk size to 1 provides a perfect way to deal with heterogeneous tasks, minimizing the idle time of threads.
    /// Each thread will retrieve elements one by one whenever they become idle.
    ///   * Considering the heterogeneous example above, the parallel execution with a chunk size of 1 would complete around 7 units of time.
    ///     * This is again the execution time of the first thread, which will only execute the first element.
    ///     * The second thread will execute the remaining 7 elements, again in 7 units in time.
    ///   * None of the threads will be idle, which is the best we can achieve.
    ///   * Drawback of this approach is the parallelization overhead due to *pull*s.
    ///   * Chunk size being 1, this setting will lead to a total of 8 pull operations (1 pull by the first thread, 7 pulls by the second thread).
    ///   * This leads to the maximum/worst parallelization overhead in this scenario.
    ///
    /// The objective then is to find a chunk size which is:
    /// * large enough that total time spent for the pulls is insignificant, while
    /// * small enough not to suffer from the impact of heterogeneity.
    ///
    /// Note that this decision is data dependent, and hence, can be tuned for the input when the operation is extremely time-critical.
    ///
    /// In these cases, the following rule of thumb helps to find a good chunk size.
    /// We can set the chunk size to the smallest value which would make the overhead of pulls insignificant:
    /// * The larger each individual task, the less significant the parallelization overhead. A small chunk size would do.
    /// * The smaller each individual task, the more significant the parallelization overhead. We require a larger chunk size while being careful not to suffer from idle times of threads due to heterogeneity.
    ///
    /// In general, it is recommended to set this parameter to its default value, `ChunkSize::Auto`.
    /// This library will try to solve the tradeoff explained above depending on the input data to minimize execution time and idle thread time.
    ///
    /// For more critical operations, this `ChunkSize::Exact` and `ChunkSize::Min` options can be used to tune the execution for the class of the relevant input data.
    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    /// Takes the closure `map` and creates an iterator which calls that closure on each element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let doubles = (0..5).into_par().map(|x| x * 2).collect_vec();
    /// assert_eq!(&doubles[..], &[0, 2, 4, 6, 8]);
    /// ```
    fn map<O, M>(self, map: M) -> impl ParIter<Item = O>
    where
        O: Send + Sync + Default,
        M: Fn(Self::Item) -> O + Send + Sync + Clone;

    /// Takes the closure `fmap` and creates an iterator which calls that closure on each element and flattens the result.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let numbers = (0..5).into_par().flat_map(|x| vec![x; x]).collect_vec();
    /// assert_eq!(&numbers[..], &[1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
    /// ```
    fn flat_map<O, OI, FM>(self, fmap: FM) -> impl ParIter<Item = O>
    where
        O: Send + Sync + Default,
        OI: IntoIterator<Item = O>,
        FM: Fn(Self::Item) -> OI + Send + Sync + Clone;

    /// Creates an iterator which uses the closure `filter` to determine if an element should be yielded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0).collect_vec();
    /// assert_eq!(&evens[..], &[0, 2, 4, 6, 8]);
    /// ```
    fn filter<F>(self, filter: F) -> impl ParIter<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync + Clone;

    /// Consumes the iterator, counting the number of iterations and returning it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0);
    /// assert_eq!(evens.count(), 5);
    /// ```
    fn count(self) -> usize;

    // find

    /// Returns the first element of the iterator satisfying the given `predicate`; returns None if the iterator is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// fn firstfac(x: usize) -> usize {
    ///     if x % 2 == 0 {
    ///         return 2;
    ///     };
    ///     for n in (1..).map(|m| 2 * m + 1).take_while(|m| m * m <= x) {
    ///         if x % n == 0 {
    ///             return n;
    ///         };
    ///     }
    ///     x
    /// }
    ///
    /// fn is_prime(n: &usize) -> bool {
    ///     match n {
    ///         0 | 1 => false,
    ///         _ => firstfac(*n) == *n,
    ///     }
    /// }
    ///
    /// let first_prime = (21..100).into_par().find(is_prime);
    /// assert_eq!(first_prime, Some(23));
    ///
    /// let first_prime = (24..28).into_par().find(is_prime);
    /// assert_eq!(first_prime, None);
    /// ```
    fn find<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Send + Sync + Clone;

    /// Returns the first element of the iterator; returns None if the iterator is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// fn firstfac(x: usize) -> usize {
    ///     if x % 2 == 0 {
    ///         return 2;
    ///     };
    ///     for n in (1..).map(|m| 2 * m + 1).take_while(|m| m * m <= x) {
    ///         if x % n == 0 {
    ///             return n;
    ///         };
    ///     }
    ///     x
    /// }
    ///
    /// fn is_prime(n: &usize) -> bool {
    ///     match n {
    ///         0 | 1 => false,
    ///         _ => firstfac(*n) == *n,
    ///     }
    /// }
    ///
    /// let first_prime = (21..100).into_par().filter(is_prime).first();
    /// assert_eq!(first_prime, Some(23));
    ///
    /// let first_prime = (24..28).into_par().filter(is_prime).first();
    /// assert_eq!(first_prime, None);
    /// ```
    fn first(self) -> Option<Self::Item>;

    // collect

    /// Transforms the iterator into a collection.
    ///
    /// In this case, the result is transformed into a standard vector; i.e., `std::vec::Vec`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0).collect_vec();
    /// assert_eq!(evens, vec![0, 2, 4, 6, 8]);
    /// ```
    fn collect_vec(self) -> Vec<Self::Item>;

    /// Transforms the iterator into a collection.
    ///
    /// In this case, the result is transformed into the split vector which is the underlying [`PinnedVec`](https://crates.io/crates/orx-pinned-vec) used to collect the results concurrently;
    /// i.e., [`SplitVec`](https://crates.io/crates/orx-split-vec).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use orx_split_vec::*;
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0).collect();
    /// assert_eq!(evens, SplitVec::from_iter([0, 2, 4, 6, 8]));
    /// ```
    fn collect(self) -> SplitVec<Self::Item>;

    /// Collects elements yielded by the iterator into the given `output` collection.
    ///
    /// Note that `output` does not need to be empty; hence, this method allows extending collections from the parallel iterator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use orx_split_vec::*;
    ///
    /// let output_vec = vec![42];
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0);
    /// let output_vec = evens.collect_into(output_vec);
    /// assert_eq!(output_vec, vec![42, 0, 2, 4, 6, 8]);
    ///
    /// let odds = (0..10).into_par().filter(|x| x % 2 == 1);
    /// let output_vec = odds.collect_into(output_vec);
    /// assert_eq!(output_vec, vec![42, 0, 2, 4, 6, 8, 1, 3, 5, 7, 9]);
    ///
    /// // alternatively, any `PinnedVec` can be used
    /// let output_vec: SplitVec<_> = [42].into_iter().collect();
    ///
    /// let evens = (0..10).into_par().filter(|x| x % 2 == 0);
    /// let output_vec = evens.collect_into(output_vec);
    /// assert_eq!(output_vec, vec![42, 0, 2, 4, 6, 8]);
    /// ```
    fn collect_into<C: ParCollectInto<Self::Item>>(self, output: C) -> C;

    /// Transforms the iterator into a collection.
    ///
    /// In this case, the result is transformed into a standard vector; i.e., `std::vec::Vec`.
    ///
    /// `collect_x_vec` differs from `collect_vec` method by the following:
    /// * `collect_vec` will  return a result which contains yielded elements in the same order. Therefore, it results in a deterministic output.
    /// `collect_x_vec`, on the other hand, does not try to preserve the order. The order of elements in the output depends on the execution speeds of different threads.
    /// * `collect_x_vec` might perform faster than `collect_vec` in certain situations.
    ///
    /// Due to above `collect_x_vec` can be preferred over `collect_vec` in performance-critical operations where the order of elements in the output is not important.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    ///
    /// let mut output = (0..5).into_par().flat_map(|x| vec![x; x]).collect_x_vec();
    /// output.sort();
    /// assert_eq!(output, vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
    /// ```
    fn collect_x_vec(self) -> Vec<Self::Item> {
        self.collect_vec()
    }

    /// Transforms the iterator into a collection.
    ///
    /// In this case, the result is transformed into the split vector which is the underlying [`PinnedVec`](https://crates.io/crates/orx-pinned-vec) used to collect the results concurrently;
    /// i.e., [`SplitVec`](https://crates.io/crates/orx-split-vec).
    ///
    /// `collect_x` differs from `collect` method by the following:
    /// * `collect` will  return a result which contains yielded elements in the same order. Therefore, it results in a deterministic output.
    /// `collect_x`, on the other hand, does not try to preserve the order. The order of elements in the output depends on the execution speeds of different threads.
    /// * `collect_x` might perform faster than `collect` in certain situations.
    ///
    /// Due to above `collect_x` can be preferred over `collect` in performance-critical operations where the order of elements in the output is not important.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use orx_split_vec::*;
    ///
    /// let output = (0..5).into_par().flat_map(|x| vec![x; x]).collect_x();
    /// let mut sorted_output = output.to_vec();
    /// sorted_output.sort(); // WIP: PinnedVec::sort(&mut self)
    /// assert_eq!(sorted_output, vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
    /// ```
    fn collect_x(self) -> SplitVec<Self::Item> {
        self.collect()
    }

    /// Collects elements yielded by the iterator into the given `output` collection.
    ///
    /// Note that `output` does not need to be empty; hence, this method allows extending collections from the parallel iterator.
    ///
    /// `collect_x_into` differs from `collect_into` method by the following:
    /// * `collect_into` will  return a result which contains yielded elements in the same order. Therefore, it results in a deterministic output.
    /// `collect_x_into`, on the other hand, does not try to preserve the order. The order of elements in the output depends on the execution speeds of different threads.
    /// * `collect_x_into` might perform faster than `collect_into` in certain situations.
    ///
    /// Due to above `collect_x_into` can be preferred over `collect_into` in performance-critical operations where the order of elements in the output is not important.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_parallel::*;
    /// use orx_split_vec::*;
    ///
    /// let mut output = SplitVec::with_doubling_growth_and_fragments_capacity(32);
    /// output.push(42);
    ///
    /// let output = (0..5).into_par().flat_map(|x| vec![x; x]).collect_x_into(output);
    /// let mut sorted_output = output.to_vec();
    /// sorted_output.sort(); // WIP: PinnedVec::sort(&mut self)
    /// assert_eq!(sorted_output, vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4, 42]);
    /// ```
    fn collect_x_into<B: ParCollectInto<Self::Item>>(self, output: B) -> B {
        self.collect_into(output)
    }
}
