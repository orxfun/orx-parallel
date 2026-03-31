/// Order of parallel iteration, which might be:
/// * in the order of the input as in regular sequential iterators, or
/// * arbitrary.
///
/// This is important for certain computations:
///
/// * `collect` will return exactly the same result of its sequential counterpart
///   when `Ordered` is used. However, the elements might (but not necessarily)
///   be in arbitrary order when `Arbitrary` is used.
/// * `first` returns the first element of the iterator when `Ordered`, might
///   return any element when `Arbitrary`.
/// * `find` returns the first element satisfying the predicate when `Ordered`,
///   might return any element satisfying the predicate when `Arbitrary`
///   (sometimes this method is called `find_any`).
///
/// [`collect`]: crate::ParIter::collect
/// [`first`]: crate::ParIter::first
/// [`find`]: crate::ParIter::find
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IterationOrder {
    /// The iteration is allowed to be in arbitrary order when it might improve performance,
    /// but not necessarily.
    Arbitrary,
    /// ***Default ordering***.
    ///
    /// The iteration will be in an order consistent with the input of the collection,
    /// and hence, the outputs will always be equivalent to the sequential counterpart.
    #[default]
    Ordered,
}
