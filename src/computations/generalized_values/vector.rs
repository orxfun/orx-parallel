use crate::computations::generalized_values::{
    whilst_atom::WhilstAtom, whilst_option::WhilstOption, whilst_vector::WhilstVector,
};

use super::values::Values;
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;

pub struct Vector<I>(pub I)
where
    I: IntoIterator;

impl<I> Values for Vector<I>
where
    I: IntoIterator,
{
    type Item = I::Item;

    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        self.0
    }

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            vector.push(x);
        }
        false
    }

    #[inline(always)]
    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) -> Option<usize> {
        for x in self.0 {
            vec.push((idx, x));
        }
        None
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> bool
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        for x in self.0 {
            bag.push(x);
        }
        false
    }

    #[inline(always)]
    fn map<M, O>(self, map: M) -> impl Values<Item = O>
    where
        M: Fn(Self::Item) -> O,
    {
        Vector(self.0.into_iter().map(map))
    }

    #[inline(always)]
    fn filter<F>(self, filter: F) -> impl Values<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool,
    {
        Vector(self.0.into_iter().filter(filter))
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> impl Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(self.0.into_iter().flat_map(flat_map))
    }

    #[inline(always)]
    fn filter_map<Fm, O>(self, filter_map: Fm) -> impl Values<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        Vector(self.0.into_iter().filter_map(filter_map))
    }

    fn whilst(self, whilst: impl Fn(&Self::Item) -> bool) -> impl Values<Item = Self::Item>
    where
        Self: Sized,
    {
        let iter = self.0.into_iter().map(move |x| match whilst(&x) {
            true => WhilstAtom::Continue(x),
            false => WhilstAtom::Stop,
        });
        WhilstVector(iter)
    }

    #[inline(always)]
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        let reduced = self.0.into_iter().reduce(&reduce);
        (
            false,
            match (acc, reduced) {
                (Some(x), Some(y)) => Some(reduce(x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            },
        )
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
    fn first(self) -> WhilstOption<Self::Item> {
        match self.0.into_iter().next() {
            Some(x) => WhilstOption::ContinueSome(x),
            None => WhilstOption::ContinueNone,
        }
    }
}
