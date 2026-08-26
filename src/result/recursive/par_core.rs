use crate::{Params, infallible::Xap, runner::ParRunner};

/// Core trait for the result (fallible) recursive parallel computation variant.
pub trait ParRecResultCore {
    /// Type of elements of the iterator.
    type Item;

    /// Type of the error.
    type Error;

    /// Type of the parallel runner.
    type Runner: ParRunner;

    /// Iterator of the initial input elements.
    type Input: IntoIterator;

    /// Intermediate type produced by the result-producing stage.
    type M;

    /// Transformation from raw input items to `Result<M, Error>`.
    type Xap1: Xap<I = <Self::Input as IntoIterator>::Item, O = Result<Self::M, Self::Error>>;

    /// Transformation from `M` to the final item.
    type Xap2: Xap<I = Self::M, O = Self::Item>;

    /// Destructs the parallel iterator into its members.
    fn destruct(self) -> (Self::Input, Self::Xap1, Self::Xap2, Self::Runner, Params);
}
