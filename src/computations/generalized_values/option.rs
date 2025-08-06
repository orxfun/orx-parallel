use super::{Values, Vector};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

impl<T> Values for Option<T>
where
    T: Send + Sync,
{
    type Item = T;

    type Mapped<M, O>
        = Option<O>
    where
        O: Send + Sync,
        M: Fn(Self::Item) -> O + Send + Sync;

    type Filtered<F>
        = Option<T>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync;

    type FlatMapped<Fm, Vo>
        = Vector<core::iter::FlatMap<<Option<T> as IntoIterator>::IntoIter, Vo, Fm>>
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

    #[inline(always)]
    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        self
    }

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P)
    where
        P: orx_fixed_vec::PinnedVec<Self::Item>,
    {
        if let Some(x) = self {
            vector.push(x)
        }
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send + Sync,
    {
        if let Some(x) = self {
            bag.push(x);
        }
    }

    #[inline(always)]
    fn push_to_ordered_bag<P>(self, idx: usize, o_bag: &ConcurrentOrderedBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send + Sync,
    {
        if let Some(x) = self {
            unsafe { o_bag.set_value(idx, x) };
        }
    }

    #[inline(always)]
    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) {
        if let Some(x) = self {
            vec.push((idx, x))
        }
    }

    #[inline(always)]
    fn map<M, O>(self, map: M) -> Self::Mapped<M, O>
    where
        O: Send + Sync,
        M: Fn(Self::Item) -> O + Send + Sync,
    {
        self.map(map)
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator + Send + Sync,
        Vo::Item: Send + Sync,
        Vo::IntoIter: Send + Sync,
        Fm: Fn(Self::Item) -> Vo + Send + Sync,
    {
        Vector(self.into_iter().flat_map(flat_map))
    }

    #[inline(always)]
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMapped<Fm, O>
    where
        O: Send + Sync,
        Fm: Fn(Self::Item) -> Option<O> + Send + Sync,
    {
        match self {
            Some(x) => filter_map(x),
            _ => None,
        }
    }

    #[inline(always)]
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match (acc, self) {
            (Some(x), Some(y)) => Some(reduce(x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }

    #[inline(always)]
    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match (acc, self) {
            (Some(x), Some(y)) => Some(reduce(u, x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }

    #[inline(always)]
    fn filter<F>(self, filter: F) -> Self::Filtered<F>
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
    {
        match self {
            Some(x) if filter(&x) => Some(x),
            _ => None,
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
        match self {
            Some(x) if filter(&x) => {
                let vo = map2(x);
                vo.acc_reduce(acc, reduce)
            }
            _ => acc,
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
        match self {
            Some(x) if filter(u, &x) => {
                let vo = map2(u, x);
                vo.u_acc_reduce(u, acc, reduce)
            }
            _ => acc,
        }
    }

    #[inline(always)]
    fn first(self) -> Option<Self::Item> {
        self
    }

    #[inline(always)]
    fn fx_next<F, M2, Vo>(self, filter: F, map2: M2) -> Option<Vo::Item>
    where
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values,
    {
        match self {
            Some(x) if filter(&x) => map2(x).first(),
            _ => None,
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
        match self {
            Some(x) if filter(u, &x) => map2(u, x).first(),
            _ => None,
        }
    }

    fn filter_map_collect_sequential<F, M2, P, Vo>(self, filter: F, map2: M2, vector: &mut P)
    where
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        match self {
            Some(x) if filter(&x) => {
                let vo = map2(x);
                vo.push_to_pinned_vec(vector);
            }
            _ => {}
        }
    }

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
        match self {
            Some(x) if filter(u, &x) => {
                let vo = map2(u, x);
                vo.push_to_pinned_vec(vector);
            }
            _ => {}
        }
    }

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
        match self {
            Some(x) if filter(&x) => {
                let vo = map2(x);
                vo.push_to_bag(bag);
            }
            _ => {}
        }
    }

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
        match self {
            Some(x) if filter(u, &x) => {
                let vo = map2(u, x);
                vo.push_to_bag(bag);
            }
            _ => {}
        }
    }

    fn xfx_collect_heap<F, M2, Vo>(
        self,
        input_idx: usize,
        filter: F,
        map2: M2,
        vec: &mut Vec<(usize, Vo::Item)>,
    ) where
        F: Fn(&Self::Item) -> bool,
        M2: Fn(Self::Item) -> Vo,
        Vo: Values,
    {
        match self {
            Some(x) if filter(&x) => {
                let vo = map2(x);
                vo.push_to_vec_with_idx(input_idx, vec);
            }
            _ => {}
        }
    }

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
        match self {
            Some(x) if filter(u, &x) => {
                let vo = map2(u, x);
                vo.push_to_vec_with_idx(input_idx, vec);
            }
            _ => {}
        }
    }

    fn filter_map_collect_in_input_order<F, M2, P, Vo>(
        self,
        input_idx: usize,
        filter: F,
        map2: M2,
        o_bag: &orx_concurrent_ordered_bag::ConcurrentOrderedBag<Vo::Item, P>,
    ) where
        F: Fn(&Self::Item) -> bool + Send + Sync,
        M2: Fn(Self::Item) -> Vo + Send + Sync,
        Vo: Values,
        Vo::Item: Send + Sync,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        match self {
            Some(x) if filter(&x) => {
                let vo = map2(x);
                vo.push_to_ordered_bag(input_idx, o_bag);
            }
            _ => {}
        }
    }
}
