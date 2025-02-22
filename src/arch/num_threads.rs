use std::num::NonZeroUsize;

/// `NumThreads` represents the degree of parallelization. It is possible to define an upper bound on the number of threads to be used for the parallel computation.
/// When set to **1**, the computation will be executed sequentially without any overhead.
/// In this sense, parallel iterators defined in this crate are a union of sequential and parallel execution.
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
///   Parallel execution will ensure that `t` will never be exceeded.
/// * We have a computation which is extremely time-critical and our benchmarks show that `t` outperforms the `NumThreads::Auto` on the corresponding system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumThreads {
    /// This setting assumes that it can use all available threads; however, the computation will spawn new threads only when required.
    /// In other words, when we can dynamically decide that the task is not large enough to justify spawning a new thread, the parallel execution will avoid it.
    Auto,
    /// Limits the maximum number of threads that can be used by the parallel execution.
    ///
    /// A special case is `NumThreads::Max(NonZeroUsize::new(1).unwrap())`, or equivalently `NumThreads::sequential()`.
    /// This will lead to a sequential execution of the defined computation on the main thread.
    /// Both in terms of used resources and computation time, this mode is not similar but **identical** to a sequential execution using the regular sequential `Iterator`s.
    ///
    /// Lastly, `NumThreads::Max(t)` where `t >= 2` can be used in the following scenarios:
    /// * We have a strict limit on the resources that we can use for this computation, even if the hardware has more resources.
    ///   Parallel execution will ensure that `t` will never be exceeded.
    /// * We have a computation which is extremely time-critical and our benchmarks show that `t` outperforms the `NumThreads::Auto` on the corresponding system.
    Max(NonZeroUsize),
}

const SEQUENTIAL: NumThreads = NumThreads::Max(unsafe { NonZeroUsize::new_unchecked(1) });

impl Default for NumThreads {
    /// Default value for number of threads is [`NumThreads::Auto`].
    fn default() -> Self {
        Self::Auto
    }
}

impl From<usize> for NumThreads {
    /// Converts the nonnegative integer to number of threads as follows:
    /// * 0 is converted to `NumThreads::Auto`,
    /// * `n` where `n > 0` is converted to `NumThreads::Max(n)`.
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Auto,
            _ => Self::Max(NonZeroUsize::new(value).expect("must be positive")),
        }
    }
}

impl NumThreads {
    /// Equivalent to `NumThreads::Max(NonZeroUsize::new(1).unwrap())`.
    ///
    /// This will lead to a sequential execution of the defined computation on the main thread.
    /// Both in terms of used resources and computation time, this mode is not similar but **identical** to a sequential execution using the regular sequential `Iterator`s.
    pub fn sequential() -> Self {
        SEQUENTIAL
    }
}
