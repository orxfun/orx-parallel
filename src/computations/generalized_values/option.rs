use super::{Values, Vector};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

impl<T> Values for Option<T> {
    type Item = T;

    type Mapped<M, O>
        = Option<O>
    where
        M: Fn(Self::Item) -> O;

    type FlatMapped<Fm, Vo>
        = Vector<core::iter::FlatMap<<Option<T> as IntoIterator>::IntoIter, Vo, Fm>>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    type FilterMapped<Fm, O>
        = Option<O>
    where
        Fm: Fn(Self::Item) -> Option<O>;

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
        Self::Item: Send,
    {
        if let Some(x) = self {
            bag.push(x);
        }
    }

    #[inline(always)]
    fn push_to_ordered_bag<P>(self, idx: usize, o_bag: &ConcurrentOrderedBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
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
        M: Fn(Self::Item) -> O,
    {
        self.map(map)
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(self.into_iter().flat_map(flat_map))
    }

    #[inline(always)]
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMapped<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
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
    fn first(self) -> Option<Self::Item> {
        self
    }
}
