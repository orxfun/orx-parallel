use crate::values::values::Values;
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;

pub trait TransformableValues: Values {
    fn map<M, O>(self, map: M) -> impl TransformableValues<Item = O>
    where
        M: Fn(Self::Item) -> O + Clone;

    fn filter<F>(self, filter: F) -> impl TransformableValues<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Clone;

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> impl TransformableValues<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo + Clone;

    fn filter_map<Fm, O>(self, filter_map: Fm) -> impl TransformableValues<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>;

    fn whilst(
        self,
        whilst: impl Fn(&Self::Item) -> bool,
    ) -> impl TransformableValues<Item = Self::Item>;
}
