use crate::infallible::ParRecIter;
use crate::infallible::xap_variants::Id;
use crate::runner::default_runner;

/// Converts dynamically expanding recursive structures into an infallible parallel iterator.
///
/// Unlike flat sources (such as slices or ranges), recursive workloads discover new items
/// while they are being processed. This trait is useful when each item can produce additional
/// items, such as traversing a tree from its root(s).
pub trait IntoParRec
where
    Self: IntoIterator + Sized,
    Self::Item: Send,
{
    fn into_par_rec<I, F>(self, extend: F) -> ParRecIter<Self, Id<Self::Item>, I, F>
    where
        I: IntoIterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> I + Send + Sync;
}

impl<S> IntoParRec for S
where
    S: IntoIterator + Sized,
    S::Item: Send,
{
    fn into_par_rec<I, F>(self, extend: F) -> ParRecIter<Self, Id<Self::Item>, I, F>
    where
        I: IntoIterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> I + Send + Sync,
    {
        ParRecIter::new(
            self,
            Id::new(),
            default_runner(),
            Default::default(),
            extend,
        )
    }
}
