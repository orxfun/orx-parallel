use super::values::Values;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub struct Vector<I>(pub I)
where
    I: IntoIterator;

impl<I> Values for Vector<I>
where
    I: IntoIterator,
{
    type Item = I::Item;

    type Mapped<M, O>
        = Vector<core::iter::Map<I::IntoIter, M>>
    where
        M: Fn(Self::Item) -> O;

    type FlatMapped<Fm, Vo>
        = Vector<core::iter::FlatMap<I::IntoIter, Vo, Fm>>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    type FilterMapped<Fm, O>
        = Vector<core::iter::FilterMap<I::IntoIter, Fm>>
    where
        Fm: Fn(Self::Item) -> Option<O>;

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
        Self::Item: Send,
    {
        for x in self.0 {
            bag.push(x);
        }
    }

    #[inline(always)]
    fn push_to_ordered_bag<P>(self, idx: usize, o_bag: &ConcurrentOrderedBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
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
        M: Fn(Self::Item) -> O,
    {
        Vector(self.0.into_iter().map(map))
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(self.0.into_iter().flat_map(flat_map))
    }

    #[inline(always)]
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMapped<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
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
    fn first(self) -> Option<Self::Item> {
        self.0.into_iter().next()
    }
}
