use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub trait Values: Send + Sync {
    type Item;

    type Mapped<M, O>: Values<Item = O>
    where
        O: Send + Sync,
        M: Fn(Self::Item) -> O + Send + Sync;

    type FlatMapped<Fm, Vo>: Values<Item = Vo::Item>
    where
        Vo: IntoIterator + Send + Sync,
        Vo::Item: Send + Sync,
        Vo::IntoIter: Send + Sync,
        Fm: Fn(Self::Item) -> Vo + Send + Sync;

    fn values(self) -> impl IntoIterator<Item = Self::Item>;

    fn push_to_pinned_vec<P>(self, vector: &mut P)
    where
        P: PinnedVec<Self::Item>;

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send + Sync;

    fn push_to_ordered_bag<P>(self, idx: usize, o_bag: &ConcurrentOrderedBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send + Sync;

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>);

    fn map<M, O>(self, map: M) -> Self::Mapped<M, O>
    where
        O: Send + Sync,
        M: Fn(Self::Item) -> O + Send + Sync;

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator + Send + Sync,
        Vo::Item: Send + Sync,
        Vo::IntoIter: Send + Sync,
        Fm: Fn(Self::Item) -> Vo + Send + Sync;

    fn filter_map_collect_sequential<F, M2, P, Vo, O>(self, filter: F, map2: M2, vector: &mut P)
    where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>;

    fn filter_map_collect_arbitrary<F, M2, P, Vo, O>(
        self,
        filter: F,
        map2: M2,
        bag: &ConcurrentBag<O, P>,
    ) where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>,
        O: Send + Sync;

    fn mfm_collect_heap<F, M2, Vo, O>(
        self,
        input_idx: usize,
        filter: F,
        map2: M2,
        vec: &mut Vec<(usize, O)>,
    ) where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        O: Send + Sync;

    fn filter_map_collect_in_input_order<F, M2, P, Vo, O>(
        self,
        input_idx: usize,
        filter: F,
        map2: M2,
        o_bag: &ConcurrentOrderedBag<O, P>,
    ) where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>,
        O: Send + Sync;

    #[cfg(test)]
    fn first(self) -> Self::Item
    where
        Self: Sized,
    {
        self.values().into_iter().next().unwrap()
    }
}
