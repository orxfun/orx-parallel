use crate::computations::{
    Values, WhilstOption, WhilstVector,
    generalized_values::whilst_iterators::WhilstOptionFlatMapIter,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub enum WhilstResult<T, E> {
    ContinueOk(T),
    StopError(E),
}

impl<T, E> Values for WhilstResult<T, E> {
    type Item = T;

    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        match self {
            Self::ContinueOk(x) => Some(x).into_iter(),
            _ => None.into_iter(),
        }
    }

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        match self {
            Self::ContinueOk(x) => {
                vector.push(x);
                false
            }
            Self::StopError(e) => true,
        }
    }

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) -> Option<usize> {
        match self {
            Self::ContinueOk(x) => {
                vec.push((idx, x));
                None
            }
            Self::StopError(e) => Some(idx),
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> bool
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self {
            Self::ContinueOk(x) => {
                bag.push(x);
                false
            }
            Self::StopError(e) => true,
        }
    }

    fn map<M, O>(self, map: M) -> impl Values<Item = O>
    where
        M: Fn(Self::Item) -> O + Clone,
    {
        match self {
            Self::ContinueOk(x) => WhilstResult::ContinueOk(map(x)),
            Self::StopError(e) => WhilstResult::StopError(e),
        }
    }

    fn filter<F>(self, filter: F) -> impl Values<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Clone,
    {
        todo!("avoid computational variant transformations all at once");
        match self {
            Self::ContinueOk(x) => match filter(&x) {
                true => Self::ContinueOk(x),
                false => todo!(
                    "we need a recursive Values definition, do we really need this? can we avoid filter?"
                ),
            },
            Self::StopError(e) => WhilstResult::StopError(e),
        }
    }

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> impl Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo + Clone,
    {
        todo!("avoid computational variant transformations all at once");
        None
    }

    fn filter_map<Fm, O>(self, filter_map: Fm) -> impl Values<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        todo!("avoid computational variant transformations all at once");
        None
    }

    fn whilst(self, whilst: impl Fn(&Self::Item) -> bool) -> impl Values<Item = Self::Item> {
        todo!("avoid computational variant transformations all at once");
        self
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueOk(x) => match acc {
                Some(acc) => (false, Some(reduce(acc, x))),
                None => (false, Some(x)),
            },
            Self::StopError(e) => (true, None), // resets entire reduction so far!
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueOk(x) => match acc {
                Some(acc) => Some(reduce(u, acc, x)),
                None => Some(x),
            },
            Self::StopError(e) => None, // resets entire reduction so far!
        }
    }

    fn first(self) -> WhilstOption<Self::Item> {
        todo!()
    }
}
