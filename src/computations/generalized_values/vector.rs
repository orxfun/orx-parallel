use super::values::Values;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub struct Vector<I>(pub I)
where
    I: IntoIterator;

impl<I> Values for Vector<I>
where
    I: IntoIterator + Send + Sync,
    I::Item: Send + Sync,
    I::IntoIter: Send + Sync,
{
    type Item = I::Item;

    type Mapped<M, O>
        = Vector<core::iter::Map<I::IntoIter, M>>
    where
        O: Send + Sync,
        M: Fn(Self::Item) -> O + Send + Sync;

    type Filtered<F>
        = Vector<core::iter::Filter<I::IntoIter, F>>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync;

    type FlatMapped<Fm, Vo>
        = Vector<core::iter::FlatMap<I::IntoIter, Vo, Fm>>
    where
        Vo: IntoIterator + Send + Sync,
        Vo::Item: Send + Sync,
        Vo::IntoIter: Send + Sync,
        Fm: Fn(Self::Item) -> Vo + Send + Sync;

    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        self.0
    }

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P)
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            vector.push(x);
        }
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send + Sync,
    {
        for x in self.0 {
            bag.push(x);
        }
    }

    #[inline(always)]
    fn push_to_ordered_bag<P>(self, idx: usize, o_bag: &ConcurrentOrderedBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send + Sync,
    {
        for x in self.0 {
            unsafe { o_bag.set_value(idx, x) }
        }
    }

    #[inline(always)]
    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) {
        for x in self.0 {
            vec.push((idx, x))
        }
    }

    #[inline(always)]
    fn map<M, O>(self, map: M) -> Self::Mapped<M, O>
    where
        O: Send + Sync,
        M: Fn(Self::Item) -> O + Send + Sync,
    {
        Vector(self.0.into_iter().map(map))
    }

    #[inline(always)]
    fn filter<F>(self, filter: F) -> Self::Filtered<F>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
    {
        Vector(self.0.into_iter().filter(filter))
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator + Send + Sync,
        Vo::Item: Send + Sync,
        Vo::IntoIter: Send + Sync,
        Fm: Fn(Self::Item) -> Vo + Send + Sync,
    {
        Vector(self.0.into_iter().flat_map(flat_map))
    }

    #[inline]
    fn filter_map_collect_sequential<F, M2, P, Vo, O>(self, filter: F, map2: M2, vector: &mut P)
    where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values<Item = O>,
        P: IntoConcurrentPinnedVec<O>,
    {
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_pinned_vec(vector);
            }
        }
    }

    #[inline]
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
        O: Send + Sync,
    {
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_bag(bag);
            }
        }
    }

    #[inline]
    fn xfx_collect_heap<F, M2, Vo, O>(
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
        O: Send + Sync,
    {
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_vec_with_idx(input_idx, vec);
            }
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
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_ordered_bag(input_idx, o_bag);
            }
        }
    }
}
