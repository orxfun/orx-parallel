use super::{values::Values, vector::Vector};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub struct Atom<T>(pub T);

impl<T> Values for Atom<T> {
    type Item = T;

    type Mapped<M, O>
        = Atom<O>
    where
        M: Fn(Self::Item) -> O;

    type Filtered<F>
        = Option<Self::Item>
    where
        F: Fn(&Self::Item) -> bool;

    type FlatMapped<Fm, Vo>
        = Vector<Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    type FilterMapped<Fm, O>
        = Option<O>
    where
        Fm: Fn(Self::Item) -> Option<O>;

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
        T: Send,
    {
        bag.push(self.0);
    }

    #[inline(always)]
    fn push_to_ordered_bag<P>(self, idx: usize, o_bag: &ConcurrentOrderedBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
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
        M: Fn(Self::Item) -> O,
    {
        Atom(map(self.0))
    }

    #[inline(always)]
    fn filter<F>(self, filter: F) -> Self::Filtered<F>
    where
        F: Fn(&Self::Item) -> bool,
    {
        match filter(&self.0) {
            true => Some(self.0),
            false => None,
        }
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(flat_map(self.0))
    }

    #[inline(always)]
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMapped<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        filter_map(self.0)
    }

    #[inline(always)]
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match acc {
            Some(x) => Some(reduce(x, self.0)),
            None => Some(self.0),
        }
    }

    #[inline(always)]
    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match acc {
            Some(x) => Some(reduce(u, x, self.0)),
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
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values,
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item,
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
        F: Fn(&mut U, &Self::Item) -> bool,
        M2: Fn(&mut U, Self::Item) -> Vo,
        Vo: Values,
        X: Fn(&mut U, Vo::Item, Vo::Item) -> Vo::Item,
    {
        match filter(u, &self.0) {
            true => map2(u, self.0).u_acc_reduce(u, acc, reduce),
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
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values,
    {
        match filter(&self.0) {
            true => map2(self.0).first(),
            false => None,
        }
    }

    #[inline(always)]
    fn u_fx_next<U, F, M2, Vo>(self, u: &mut U, filter: F, map2: M2) -> Option<Vo::Item>
    where
        F: Fn(&mut U, &Self::Item) -> bool,
        M2: Fn(&mut U, Self::Item) -> Vo,
        Vo: Values,
    {
        match filter(u, &self.0) {
            true => map2(u, self.0).first(),
            false => None,
        }
    }

    #[inline(always)]
    fn filter_map_collect_sequential<F, M2, P, Vo>(self, filter: F, map2: M2, vector: &mut P)
    where
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
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
        F: Fn(&mut U, &Self::Item) -> bool,
        M2: Fn(&mut U, Self::Item) -> Vo,
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
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values,
        Vo::Item: Send,
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
        F: Fn(&mut U, &Self::Item) -> bool,
        M2: Fn(&mut U, Self::Item) -> Vo,
        Vo: Values,
        P: IntoConcurrentPinnedVec<Vo::Item>,
        Vo::Item: Send,
    {
        if filter(u, &self.0) {
            let vo = map2(u, self.0);
            vo.push_to_bag(bag);
        }
    }
}
