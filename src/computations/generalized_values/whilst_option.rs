use crate::computations::{
    Values, WhilstVector, generalized_values::whilst_iterators::WhilstOptionFlatMapIter,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub enum WhilstOption<T> {
    ContinueSome(T),
    ContinueNone,
    Stop,
}

impl<T> Values for WhilstOption<T> {
    type Item = T;

    type Error = ();

    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        match self {
            Self::ContinueSome(x) => Some(x).into_iter(),
            _ => None.into_iter(),
        }
    }

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        match self {
            Self::ContinueSome(x) => {
                vector.push(x);
                false
            }
            Self::ContinueNone => false,
            Self::Stop => true,
        }
    }

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) -> Option<usize> {
        match self {
            Self::ContinueSome(x) => {
                vec.push((idx, x));
                None
            }
            Self::ContinueNone => None,
            Self::Stop => Some(idx),
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> bool
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self {
            Self::ContinueSome(x) => {
                bag.push(x);
                false
            }
            Self::ContinueNone => false,
            Self::Stop => true,
        }
    }

    fn map<M, O>(self, map: M) -> impl Values<Item = O>
    where
        M: Fn(Self::Item) -> O + Clone,
    {
        match self {
            Self::ContinueSome(x) => WhilstOption::ContinueSome(map(x)),
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
        }
    }

    fn filter<F>(self, filter: F) -> impl Values<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Clone,
    {
        match self {
            Self::ContinueSome(x) => match filter(&x) {
                true => Self::ContinueSome(x),
                false => Self::ContinueNone,
            },
            Self::ContinueNone => Self::ContinueNone,
            Self::Stop => Self::Stop,
        }
    }

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> impl Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo + Clone,
    {
        let iter = WhilstOptionFlatMapIter::from_option(self, &flat_map);
        WhilstVector(iter)
    }

    fn filter_map<Fm, O>(self, filter_map: Fm) -> impl Values<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        match self {
            Self::ContinueSome(x) => match filter_map(x) {
                Some(x) => WhilstOption::ContinueSome(x),
                None => WhilstOption::ContinueNone,
            },
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
        }
    }

    fn whilst(self, whilst: impl Fn(&Self::Item) -> bool) -> impl Values<Item = Self::Item>
    where
        Self: Sized,
    {
        match self {
            Self::ContinueSome(x) => match whilst(&x) {
                true => Self::ContinueSome(x),
                false => Self::Stop,
            },
            Self::ContinueNone => Self::ContinueNone,
            Self::Stop => Self::Stop,
        }
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueSome(x) => match acc {
                Some(acc) => (false, Some(reduce(acc, x))),
                None => (false, Some(x)),
            },
            Self::ContinueNone => (false, acc),
            Self::Stop => (true, acc),
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueSome(x) => match acc {
                Some(acc) => Some(reduce(u, acc, x)),
                None => Some(x),
            },
            Self::ContinueNone => acc,
            Self::Stop => acc,
        }
    }

    fn first(self) -> WhilstOption<Self::Item> {
        self
    }
}
