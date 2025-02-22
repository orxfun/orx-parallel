use crate::{chunk_size::ChunkSize, num_threads::NumThreads};

/// Parallelization parameters consisting of two settings.
/// * `num_threads` represents the degree of parallelization. It is possible to define an upper bound on the number of threads to be used for the parallel computation.
///   When set to **1**, the computation will be executed **sequentially without any overhead**.
///   In this set, parallel iterators defined in this crate are a union of sequential and parallel execution.
/// * `chunk_size` represents the batch size of elements each thread will pull from the main iterator once it becomes idle again.
///   It is possible to define a minimum or exact chunk size.
///
/// When not set, or explicitly set to **Auto**, this crate will dynamically decide their values with the following two goals:
/// * complete the work as fast as possible,
/// * do not use unnecessary resources; i.e., do not spawn any unnecessary threads, if the overhead of parallelization overweighs the gain of it.
///
/// # Examples
///
/// ```rust
/// use orx_parallel::*;
/// use std::num::NonZeroUsize;
///
/// let params = Params::default();
/// assert_eq!(params.num_threads, NumThreads::Auto);
/// assert_eq!(params.chunk_size, ChunkSize::Auto);
///
/// let params = Params {
///     num_threads: NumThreads::Max(NonZeroUsize::new(4).unwrap()),
///     chunk_size: ChunkSize::Min(NonZeroUsize::new(64).unwrap()),
/// };
/// assert_eq!(params.num_threads, NumThreads::Max(NonZeroUsize::new(4).unwrap()));
/// assert_eq!(params.chunk_size, ChunkSize::Min(NonZeroUsize::new(64).unwrap()));
///
/// let params = Params {
///     num_threads: 8.into(), // positive num threads maps to NumThreads::Max
///     chunk_size: 32.into(), // positive chunk size maps to ChunkSize::Exact
/// };
/// assert_eq!(params.num_threads, 8.into());
/// assert_eq!(params.chunk_size, 32.into());
///
/// let params = Params {
///     num_threads: 0.into(), // zero num threads maps to NumThreads::Auto
///     chunk_size: 0.into(),  // zero chunk size maps to ChunkSize::Auto
/// };
/// assert_eq!(params.num_threads, NumThreads::Auto);
/// assert_eq!(params.chunk_size, ChunkSize::Auto);
///
/// let params = Params {
///     num_threads: 8.into(),
///     chunk_size: ChunkSize::Min(NonZeroUsize::new(64).unwrap()), // ChunkSize::Min requires setting explicitly
/// };
/// assert_eq!(params.num_threads, 8.into());
/// assert_eq!(params.chunk_size, ChunkSize::Min(NonZeroUsize::new(64).unwrap()));
/// ```
///
/// # Rules of Thumb / Guidelines
///
/// This crate boils down the complexity of parallel computing into two simple and straightforward parameters.
///
/// ## NumThreads
///
/// It is recommended to set this parameter to its default value, `NumThreads::Auto`.
/// This setting assumes that it can use all available threads; however, the computation will spawn new threads only when required.
/// In other words, when it can dynamically decide that the task is not large enough to justify spawning a new thread, the parallel execution will avoid it.
///
/// A special case is `NumThreads::Max(NonZeroUsize::new(1).unwrap())`, or equivalently `NumThreads::sequential()`.
/// This will lead to a sequential execution of the defined computation on the main thread.
/// Both in terms of used resources and computation time, this mode is **identical** to a sequential execution using the regular sequential `Iterator`s.
///
/// Lastly, `NumThreads::Max(t)` where `t >= 2` can be used in the following scenarios:
/// * We have a strict limit on the resources that we can use for this computation, even if the hardware has more resources.
///   Parallel execution will ensure that `t` will never be exceeded.
/// * We have a computation which is extremely time-critical and our benchmarks show that `t` outperforms the `NumThreads::Auto` on the corresponding system.
///
/// ## ChunkSize
///
/// The objective of this parameter is to balance the overhead of parallelization and cost of heterogeneity of tasks.
///
/// In order to illustrate, assume that there exist 8 elements to process, or 8 jobs to execute, and we will use 2 threads for this computation.
/// Two extreme strategies can be defined as follows.
///
/// * **Perfect Sharing of Tasks**
///   * Setting chunk size to 4 provides a perfect division of tasks in terms of quantity.
///     Each thread will retrieve 4 elements at once in one pull and process them.
///     This *one pull* per thread can be considered as the parallelization overhead and this is the best/minimum we can achieve.
///   * Drawback of this approach, on the other hand, is observed when the execution time of each job is significantly different; i.e., when we have heterogeneous tasks.
///   * Assume, for instance, that the first element requires 7 units of time while all remaining elements require 1 unit of time.
///   * Roughly, the parallel execution with a chunk size of 4 would complete in 10 units of time, which is the execution time of the first thread (7 + 3*1).
///   * The second thread will complete its 4 tasks in 4 units of time and will remain idle for 6 units of time.
/// * **Perfect Handling of Heterogeneity**
///   * Setting chunk size to 1 provides a perfect way to deal with heterogeneous tasks, minimizing the idle time of threads.
///     Each thread will retrieve elements one by one whenever they become idle.
///   * Considering the heterogeneous example above, the parallel execution with a chunk size of 1 would complete around 7 units of time.
///     * This is again the execution time of the first thread, which will only execute the first element.
///     * The second thread will execute the remaining 7 elements, again in 7 units in time.
///   * None of the threads will be idle, which is the best we can achieve.
///   * Drawback of this approach is the parallelization overhead due to *pull*s. This setting will lead to a total of 8 pull operations (1 pull by the first thread, 7 pulls by the second thread).
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params {
    /// `num_threads` represents the degree of parallelization. It is possible to define an upper bound on the number of threads to be used for the parallel computation.
    /// When set to **1**, the computation will be executed sequentially without any overhead.
    /// In this sense, parallel iterators defined in this crate are a union of sequential and parallel execution.
    ///
    /// # Rules of Thumb / Guidelines
    ///
    /// It is recommended to set this parameter to its default value, `NumThreads::Auto`.
    /// This setting assumes that it can use all available threads; however, the computation will spawn new threads only when required.
    /// In other words, when it can dynamically decide that the task is not large enough to justify spawning a new thread, the parallel execution will avoid it.
    ///
    /// A special case is `NumThreads::Max(NonZeroUsize::new(1).unwrap())`, or equivalently `NumThreads::sequential()`.
    /// This will lead to a sequential execution of the defined computation on the main thread.
    /// Both in terms of used resources and computation time, this mode is **identical** to a sequential execution using the regular sequential `Iterator`s.
    ///
    /// Lastly, `NumThreads::Max(t)` where `t >= 2` can be used in the following scenarios:
    /// * We have a strict limit on the resources that we can use for this computation, even if the hardware has more resources.
    ///   Parallel execution will ensure that `t` will never be exceeded.
    /// * We have a computation which is extremely time-critical and our benchmarks show that `t` outperforms the `NumThreads::Auto` on the corresponding system.
    pub num_threads: NumThreads,
    /// `chunk_size` represents the batch size of elements each thread will pull from the main iterator once it becomes idle again.
    /// It is possible to define a minimum or exact chunk size.
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
    ///     Each thread will retrieve 4 elements at once in one pull and process them.
    ///     This *one pull* per thread can be considered as the parallelization overhead and this is the best/minimum we can achieve.
    ///   * Drawback of this approach, on the other hand, is observed when the execution time of each job is significantly different; i.e., when we have heterogeneous tasks.
    ///   * Assume, for instance, that the first element requires 7 units of time while all remaining elements require 1 unit of time.
    ///   * Roughly, the parallel execution with a chunk size of 4 would complete in 10 units of time, which is the execution time of the first thread (7 + 3*1).
    ///   * The second thread will complete its 4 tasks in 4 units of time and will remain idle for 6 units of time.
    /// * **Perfect Handling of Heterogeneity**
    ///   * Setting chunk size to 1 provides a perfect way to deal with heterogeneous tasks, minimizing the idle time of threads.
    ///     Each thread will retrieve elements one by one whenever they become idle.
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
    pub chunk_size: ChunkSize,
}

impl Params {
    /// Returns whether or not the parameters are set to sequential execution.
    ///
    /// This is equivalent to checking if the number of threads is set to 1; i.e., `self.num_threads == Self::Max(NonZeroUsize::new(1).unwrap())` or `self.num_threads == NumThreads::sequential()`.
    pub fn is_sequential(self) -> bool {
        self.num_threads == NumThreads::sequential()
    }

    pub(crate) fn with_num_threads(self, num_threads: impl Into<NumThreads>) -> Self {
        Self {
            num_threads: num_threads.into(),
            chunk_size: self.chunk_size,
        }
    }

    pub(crate) fn with_chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self {
        Self {
            num_threads: self.num_threads,
            chunk_size: chunk_size.into(),
        }
    }
}
