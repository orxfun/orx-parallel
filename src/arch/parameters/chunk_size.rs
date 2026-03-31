use core::num::NonZeroUsize;

/// `ChunkSize` represents the batch size of elements each thread will pull from the main iterator once it becomes idle again.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChunkSize {
    /// The objective of `ChunkSize` parameter is to balance the overhead of parallelization and cost of heterogeneity of tasks.
    ///
    /// When `ChunkSize::Auto` is used, this library will try to solve the tradeoff explained above depending on the input data to minimize execution time and idle thread time.
    #[default]
    Auto,
    /// This variant defines a minimum chunk size, say `min_chunk_size`.
    /// Each time a thread completes a task and becomes idle, it will pull at least `min_chunk_size` elements from the input source.
    /// Parallel execution is allowed to and might decide to pull more elements depending on characteristics of the inputs and used number of threads.
    Min(NonZeroUsize),
    /// This variant defines an exact chunk size, say `exact_chunk_size`.
    /// Each time a thread completes a task and becomes idle, it will pull exactly `exact_chunk_size` elements from the input source.
    Exact(NonZeroUsize),
}

impl From<usize> for ChunkSize {
    /// Converts the nonnegative integer to chunk size as follows:
    ///
    /// * 0 is converted to `ChunkSize::Auto`,
    /// * `n` is converted to `ChunkSize::Exact(n)` where `n > 0`.
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Auto,
            n => Self::Exact(NonZeroUsize::new(n).expect("is positive")),
        }
    }
}
