use crate::{Params, infallible::Xap, runner::ParRunner};
use orx_concurrent_iter::ConcurrentIter;

/// Core trait for all parallel computation variants.
pub trait ParCore {
    /// Type of elements of the iterator.
    type Item;

    /// Type of the parallel runner.
    type Runner: ParRunner;

    /// Concurrent iterator of the input elements.
    type Input: ConcurrentIter;

    /// Transformation.
    type Xap: Xap<I = <Self::Input as ConcurrentIter>::Item, O = Self::Item>;

    /// Destructs the parallel iterator into its members.
    fn destruct(self) -> (Self::Input, Self::Xap, Self::Runner, Params);

    fn get_runner(&self) -> &Self::Runner;
}
