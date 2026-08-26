use crate::{Params, infallible::Xap, runner::ParRunner};

/// Core trait for all parallel computation variants.
pub trait ParRecCore {
    /// Type of elements of the iterator.
    type Item;

    /// Type of the parallel runner.
    type Runner: ParRunner;

    /// Iterator of the initial input elements.
    type Input: IntoIterator;

    /// Transformation.
    type Xap: Xap<I = <Self::Input as IntoIterator>::Item, O = Self::Item>;

    /// Destructs the parallel iterator into its members.
    fn destruct(self) -> (Self::Input, Self::Xap, Self::Runner, Params);
}
