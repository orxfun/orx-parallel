use super::{values::Values, vector::Vector};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub struct Atom<T>(pub T);

impl<T> Values for Atom<T>
where
    T: Send + Sync,
{
    type Item = T;

    type Mapped<M, O>
        = Atom<O>
    where
        O: Send + Sync,
        M: Fn(Self::Item) -> O + Send + Sync;

    type Filtered<F>
        = Vector<Option<Self::Item>>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync;

    type FlatMapped<Fm, Vo>
        = Vector<Vo>
    where
        Vo: IntoIterator + Send + Sync,
        Vo::Item: Send + Sync,
        Vo::IntoIter: Send + Sync,
        Fm: Fn(Self::Item) -> Vo + Send + Sync;

    fn values(self) -> impl IntoIterator<Item = T> {
        core::iter::once(self.0)
    }

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P)
    where
        P: PinnedVec<T>,
    {
        vector.push(self.0);
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<T, P>)
    where
        P: IntoConcurrentPinnedVec<T>,
        T: Send + Sync,
    {
        bag.push(self.0);
    }

    #[inline(always)]
    fn push_to_ordered_bag<P>(self, idx: usize, o_bag: &ConcurrentOrderedBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send + Sync,
    {
        unsafe { o_bag.set_value(idx, self.0) };
    }

    #[inline(always)]
    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, T)>) {
        vec.push((idx, self.0))
    }

    #[inline(always)]
    fn map<M, O>(self, map: M) -> Self::Mapped<M, O>
    where
        O: Send + Sync,
        M: Fn(Self::Item) -> O + Send + Sync,
    {
        Atom(map(self.0))
    }

    #[inline(always)]
    fn filter<F>(self, filter: F) -> Self::Filtered<F>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
    {
        Vector(match filter(&self.0) {
            true => Some(self.0),
            false => None,
        })
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator + Send + Sync,
        Vo::Item: Send + Sync,
        Vo::IntoIter: Send + Sync,
        Fm: Fn(Self::Item) -> Vo + Send + Sync,
    {
        Vector(flat_map(self.0))
    }

    #[inline(always)]
    fn filter_map_collect_sequential<F, M2, P, Vo, O>(self, filter: F, map2: M2, vector: &mut P)
    where
        Self: Sized,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_pinned_vec(vector);
        }
    }

    #[inline(always)]
    fn filter_map_collect_arbitrary<F, M2, P, Vo, O>(
        self,
        filter: F,
        map2: M2,
        bag: &ConcurrentBag<O, P>,
    ) where
        Self: Sized,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>,
        O: Send + Sync,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_bag(bag);
        }
    }

    #[inline(always)]
    fn xfx_collect_heap<F, M2, Vo, O>(
        self,
        input_idx: usize,
        filter: F,
        map2: M2,
        vec: &mut Vec<(usize, O)>,
    ) where
        Self: Sized,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        O: Send + Sync,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_vec_with_idx(input_idx, vec);
        }
    }

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
        O: Send + Sync,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_ordered_bag(input_idx, o_bag);
        }
    }
}
