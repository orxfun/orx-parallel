use crate::computations::{Values, Vector};
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub struct WhileOption<T>(Option<T>);

impl<T> IntoIterator for WhileOption<T> {
    type Item = T;

    type IntoIter = <Option<T> as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Values for WhileOption<T> {
    type Item = T;

    #[inline(always)]
    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        todo!();
        self.0
    }

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P) -> Option<usize>
    where
        P: PinnedVec<Self::Item>,
    {
        match self.0 {
            Some(x) => {
                vector.push(x);
                None
            }
            None => todo!(),
        }
    }

    #[inline(always)]
    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) -> Option<usize> {
        if let Some(x) = self.0 {
            vec.push((idx, x));
        }
        None
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> Option<usize>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        if let Some(x) = self.0 {
            bag.push(x);
        }
        None
    }

    #[inline(always)]
    fn map<M, O>(self, map: M) -> impl Values<Item = O>
    where
        M: Fn(Self::Item) -> O,
    {
        self.0.map(map)
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
        match self.0 {
            Some(x) => filter_map(x),
            _ => None,
        }
    }

    #[inline(always)]
    fn acc_reduce<X>(
        self,
        acc: Option<Self::Item>,
        reduce: X,
    ) -> (Option<usize>, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        (
            None,
            match (acc, self.0) {
                (Some(x), Some(y)) => Some(reduce(x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            },
        )
    }

    #[inline(always)]
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

    #[inline(always)]
    fn first(self) -> Option<Self::Item> {
        self.0
    }
}
