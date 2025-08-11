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

impl<T> WhileOption<T> {
    #[inline(always)]
    pub fn new(value: T, until: impl Fn(&T) -> bool) -> Self {
        match until(&value) {
            true => Self::continue_with(value),
            false => Self::stop(),
        }
    }

    #[inline(always)]
    pub fn stop() -> Self {
        Self(None)
    }

    #[inline(always)]
    pub fn continue_with(value: T) -> Self {
        Self(Some(value))
    }

    #[inline(always)]
    pub fn is_stop(&self) -> bool {
        self.0.is_none()
    }

    #[inline(always)]
    pub fn is_continue(&self) -> bool {
        self.0.is_some()
    }

    #[inline(always)]
    pub fn into_continue(self) -> Option<T> {
        self.0
    }

    #[inline(always)]
    pub fn mapped<O>(self, map: impl Fn(T) -> O) -> WhileOption<O> {
        WhileOption(self.0.map(map))
    }

    #[inline(always)]
    pub fn filtered(self, filter: impl Fn(&T) -> bool) -> Option<Self> {
        match &self.0 {
            Some(x) => match filter(x) {
                true => Some(self),
                false => None,
            },
            None => Some(self),
        }
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
    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        match self.0 {
            Some(x) => {
                vector.push(x);
                false
            }
            None => true,
        }
    }

    #[inline(always)]
    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) -> Option<usize> {
        match self.0 {
            Some(x) => {
                vec.push((idx, x));
                None
            }
            None => Some(idx),
        }
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> bool
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self.0 {
            Some(x) => {
                bag.push(x);
                false
            }
            None => true,
        }
    }

    #[inline(always)]
    fn map<M, O>(self, map: M) -> impl Values<Item = O>
    where
        M: Fn(Self::Item) -> O,
    {
        WhileOption(self.0.map(map))
    }

    #[inline(always)]
    fn filter<F>(self, filter: F) -> impl Values<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool,
    {
        WhileOption(self.0.filter(filter))
    }

    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> impl Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        todo!();
        Vector(self.0.into_iter().flat_map(flat_map))
    }

    #[inline(always)]
    fn filter_map<Fm, O>(self, filter_map: Fm) -> impl Values<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        match self.0 {
            Some(x) => WhileOption(filter_map(x)),
            _ => WhileOption(None),
        }
    }

    #[inline(always)]
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match (acc, self.0) {
            (Some(x), Some(y)) => (false, Some(reduce(x, y))),
            (None, Some(y)) => (false, Some(y)),
            (Some(x), None) => (true, Some(x)),
            (None, None) => (true, None),
        }
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
