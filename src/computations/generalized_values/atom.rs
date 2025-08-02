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
        = Option<Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync;

    type FlatMapped<Fm, Vo>
        = Vector<Vo>
    where
        Vo: IntoIterator + Send + Sync,
        Vo::Item: Send + Sync,
        Vo::IntoIter: Send + Sync,
        Fm: Fn(Self::Item) -> Vo + Send + Sync;

    type FilterMapped<Fm, O>
        = Option<O>
    where
        O: Send + Sync,
        Fm: Fn(Self::Item) -> Option<O> + Send + Sync;

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
        match filter(&self.0) {
            true => Some(self.0),
            false => None,
        }
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
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMapped<Fm, O>
    where
        O: Send + Sync,
        Fm: Fn(Self::Item) -> Option<O> + Send + Sync,
    {
        filter_map(self.0)
    }

    #[inline(always)]
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync,
    {
        match acc {
            Some(x) => Some(reduce(x, self.0)),
            None => Some(self.0),
        }
    }

    #[inline(always)]
    fn fx_reduce<F, M2, Vo, X>(
        self,
        acc: Option<Vo::Item>,
        filter: F,
        map2: M2,
        reduce: X,
    ) -> Option<Vo::Item>
    where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        match filter(&self.0) {
            true => map2(self.0).acc_reduce(acc, reduce),
            false => acc,
        }
    }

    #[inline(always)]
    fn u_fx_reduce<U, F, M2, Vo, X>(
        self,
        u: &mut U,
        acc: Option<Vo::Item>,
        filter: F,
        map2: M2,
        reduce: X,
    ) -> Option<Vo::Item>
    where
        Self: Sized,
        F: Fn(&mut U, &Self::Item) -> bool + Send + Sync,
        M2: Fn(&mut U, Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        match filter(u, &self.0) {
            true => map2(u, self.0).acc_reduce(acc, reduce),
            false => acc,
        }
    }

    #[inline(always)]
    fn first(self) -> Option<Self::Item> {
        Some(self.0)
    }

    #[inline(always)]
    fn fx_next<F, M2, Vo>(self, filter: F, map2: M2) -> Option<Vo::Item>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
    {
        match filter(&self.0) {
            true => map2(self.0).first(),
            false => None,
        }
    }

    #[inline(always)]
    fn u_fx_next<U, F, M2, Vo>(self, u: &mut U, filter: F, map2: M2) -> Option<Vo::Item>
    where
        F: Fn(&mut U, &Self::Item) -> bool + Send + Sync,
        M2: Fn(&mut U, Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
    {
        match filter(u, &self.0) {
            true => map2(u, self.0).first(),
            false => None,
        }
    }

    #[inline(always)]
    fn filter_map_collect_sequential<F, M2, P, Vo>(self, filter: F, map2: M2, vector: &mut P)
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_pinned_vec(vector);
        }
    }

    #[inline(always)]
    fn u_filter_map_collect_sequential<U, F, M2, P, Vo>(
        self,
        u: &mut U,
        filter: F,
        map2: M2,
        vector: &mut P,
    ) where
        F: Fn(&mut U, &Self::Item) -> bool + Send + Sync,
        M2: Fn(&mut U, Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        if filter(u, &self.0) {
            let vo = map2(u, self.0);
            vo.push_to_pinned_vec(vector);
        }
    }

    #[inline(always)]
    fn filter_map_collect_arbitrary<F, M2, P, Vo>(
        self,
        filter: F,
        map2: M2,
        bag: &ConcurrentBag<Vo::Item, P>,
    ) where
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_bag(bag);
        }
    }

    #[inline(always)]
    fn u_filter_map_collect_arbitrary<U, F, M2, P, Vo>(
        self,
        u: &mut U,
        filter: F,
        map2: M2,
        bag: &ConcurrentBag<Vo::Item, P>,
    ) where
        F: Fn(&mut U, &Self::Item) -> bool + Send + Sync,
        M2: Fn(&mut U, Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        if filter(u, &self.0) {
            let vo = map2(u, self.0);
            vo.push_to_bag(bag);
        }
    }

    #[inline(always)]
    fn xfx_collect_heap<F, M2, Vo>(
        self,
        input_idx: usize,
        filter: F,
        map2: M2,
        vec: &mut Vec<(usize, Vo::Item)>,
    ) where
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_vec_with_idx(input_idx, vec);
        }
    }

    #[inline(always)]
    fn u_xfx_collect_heap<U, F, M2, Vo>(
        self,
        u: &mut U,
        input_idx: usize,
        filter: F,
        map2: M2,
        vec: &mut Vec<(usize, Vo::Item)>,
    ) where
        F: Fn(&mut U, &Self::Item) -> bool + Send + Sync,
        M2: Fn(&mut U, Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
    {
        if filter(u, &self.0) {
            let vo = map2(u, self.0);
            vo.push_to_vec_with_idx(input_idx, vec);
        }
    }

    fn filter_map_collect_in_input_order<F, M2, P, Vo>(
        self,
        input_idx: usize,
        filter: F,
        map2: M2,
        o_bag: &ConcurrentOrderedBag<Vo::Item, P>,
    ) where
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        if filter(&self.0) {
            let vo = map2(self.0);
            vo.push_to_ordered_bag(input_idx, o_bag);
        }
    }
}
