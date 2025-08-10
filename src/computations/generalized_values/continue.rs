use crate::computations::{Values, Vector};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub struct Continue<T>(Option<T>);

impl<T> IntoIterator for Continue<T> {
    type Item = T;

    type IntoIter = <Option<T> as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Values for Continue<T> {
    type Item = T;

    type Mapped<M, O>
        = Continue<O>
    where
        M: Fn(Self::Item) -> O;

    type Filtered<F>
        = Continue<T>
    where
        F: Fn(&Self::Item) -> bool;

    type FlatMapped<Fm, Vo>
        = Vector<core::iter::FlatMap<<Continue<T> as IntoIterator>::IntoIter, Vo, Fm>>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;

    type FilterMapped<Fm, O>
        = Continue<O>
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
        if let Some(x) = self.0 {
            vector.push(x)
        }
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        if let Some(x) = self.0 {
            bag.push(x);
        }
    }

    fn push_to_ordered_bag<P>(self, idx: usize, o_bag: &ConcurrentOrderedBag<Self::Item, P>)
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        if let Some(x) = self.0 {
            unsafe { o_bag.set_value(idx, x) };
        }
    }

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) {
        if let Some(x) = self.0 {
            vec.push((idx, x))
        }
    }

    fn map<M, O>(self, map: M) -> Self::Mapped<M, O>
    where
        M: Fn(Self::Item) -> O,
    {
        Continue(self.0.map(map))
    }

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMapped<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(self.into_iter().flat_map(flat_map))
    }

    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMapped<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        Continue(match self.0 {
            Some(x) => filter_map(x),
            _ => None,
        })
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match (acc, self.0) {
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
        match (acc, self.0) {
            (Some(x), Some(y)) => Some(reduce(u, x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        }
    }

    fn first(self) -> Option<Self::Item> {
        self.0
    }
}
