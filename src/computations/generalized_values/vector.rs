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

    type FilterMapped<Fm, O>
        = Vector<core::iter::FilterMap<I::IntoIter, Fm>>
    where
        O: Send + Sync,
        Fm: Fn(Self::Item) -> Option<O> + Send + Sync;

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

    #[inline(always)]
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMapped<Fm, O>
    where
        O: Send + Sync,
        Fm: Fn(Self::Item) -> Option<O> + Send + Sync,
    {
        Vector(self.0.into_iter().filter_map(filter_map))
    }

    #[inline(always)]
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        let reduced = self.0.into_iter().reduce(&reduce);
        match (acc, reduced) {
            (Some(x), Some(y)) => Some(reduce(x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        let reduced = self.0.into_iter().reduce(|a, b| reduce(u, a, b));
        match (acc, reduced) {
            (Some(x), Some(y)) => Some(reduce(u, x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }

    #[inline(always)]
    fn fx_reduce<F, M2, Vo, X>(
        self,
        mut acc: Option<Vo::Item>,
        filter: F,
        map2: M2,
        reduce: X,
    ) -> Option<Vo::Item>
    where
        Self: Sized,
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send,
        X: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                acc = vo.acc_reduce(acc, &reduce);
            }
        }

        acc
    }

    #[inline(always)]
    fn u_fx_reduce<U, F, M2, Vo, X>(
        self,
        u: &mut U,
        mut acc: Option<Vo::Item>,
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
        X: Fn(&mut U, Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        for t in self.0 {
            if filter(u, &t) {
                let vo = map2(u, t);
                acc = vo.u_acc_reduce(u, acc, &reduce);
            }
        }

        acc
    }

    #[inline(always)]
    fn first(self) -> Option<Self::Item> {
        self.0.into_iter().next()
    }

    #[inline(always)]
    fn fx_next<F, M2, Vo>(self, filter: F, map2: M2) -> Option<Vo::Item>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send,
    {
        self.0
            .into_iter()
            .filter(filter)
            .filter_map(|t| map2(t).first())
            .next()
    }

    fn u_fx_next<U, F, M2, Vo>(self, u: &mut U, filter: F, map2: M2) -> Option<Vo::Item>
    where
        F: Fn(&mut U, &Self::Item) -> bool + Send + Sync,
        M2: Fn(&mut U, Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
    {
        // TODO: avoid intermediate collection
        let x = self
            .0
            .into_iter()
            .filter(|x| filter(u, x))
            .collect::<Vec<_>>();
        x.into_iter().filter_map(|t| map2(u, t).first()).next()
    }

    #[inline]
    fn filter_map_collect_sequential<F, M2, P, Vo>(self, filter: F, map2: M2, vector: &mut P)
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_pinned_vec(vector);
            }
        }
    }

    #[inline]
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
        for t in self.0 {
            if filter(u, &t) {
                let vo = map2(u, t);
                vo.push_to_pinned_vec(vector);
            }
        }
    }

    #[inline]
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
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_bag(bag);
            }
        }
    }

    #[inline]
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
        for t in self.0 {
            if filter(u, &t) {
                let vo = map2(u, t);
                vo.push_to_bag(bag);
            }
        }
    }

    #[inline]
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
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_vec_with_idx(input_idx, vec);
            }
        }
    }

    #[inline]
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
        for t in self.0 {
            if filter(u, &t) {
                let vo = map2(u, t);
                vo.push_to_vec_with_idx(input_idx, vec);
            }
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
        for t in self.0 {
            if filter(&t) {
                let vo = map2(t);
                vo.push_to_ordered_bag(input_idx, o_bag);
            }
        }
    }
}
