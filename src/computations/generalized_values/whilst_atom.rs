use crate::computations::{
    Values, WhilstVector,
    generalized_values::{
        Never, whilst_iterators::WhilstAtomFlatMapIter, whilst_option::WhilstOption,
    },
};
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub enum WhilstAtom<T> {
    Continue(T),
    Stop,
}

impl<T> WhilstAtom<T> {
    #[inline(always)]
    pub fn new(value: T, whilst: impl Fn(&T) -> bool) -> Self {
        match whilst(&value) {
            true => Self::Continue(value),
            false => Self::Stop,
        }
    }
}

impl<T> Values for WhilstAtom<T> {
    type Item = T;

    type Error = Never;

    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        match self {
            Self::Continue(x) => Some(x).into_iter(),
            _ => None.into_iter(),
        }
    }

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        match self {
            Self::Continue(x) => {
                vector.push(x);
                false
            }
            Self::Stop => true,
        }
    }

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) -> Option<usize> {
        match self {
            Self::Continue(x) => {
                vec.push((idx, x));
                None
            }
            Self::Stop => Some(idx),
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> bool
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self {
            Self::Continue(x) => {
                bag.push(x);
                false
            }
            Self::Stop => true,
        }
    }

    fn map<M, O>(self, map: M) -> impl Values<Item = O>
    where
        M: Fn(Self::Item) -> O + Clone,
    {
        match self {
            Self::Continue(x) => WhilstAtom::Continue(map(x)),
            Self::Stop => WhilstAtom::Stop,
        }
    }

    fn filter<F>(self, filter: F) -> impl Values<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Clone,
    {
        match self {
            Self::Continue(x) => match filter(&x) {
                true => WhilstOption::ContinueSome(x),
                false => WhilstOption::ContinueNone,
            },
            Self::Stop => WhilstOption::Stop,
        }
    }

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> impl Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo + Clone,
    {
        let iter = WhilstAtomFlatMapIter::from_atom(self, &flat_map);
        WhilstVector(iter)
    }

    fn filter_map<Fm, O>(self, filter_map: Fm) -> impl Values<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        match self {
            Self::Continue(x) => match filter_map(x) {
                Some(x) => WhilstOption::ContinueSome(x),
                None => WhilstOption::ContinueNone,
            },
            Self::Stop => WhilstOption::Stop,
        }
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::Continue(x) => match acc {
                Some(acc) => (false, Some(reduce(acc, x))),
                None => (false, Some(x)),
            },
            Self::Stop => (true, acc),
        }
    }

    fn whilst(self, whilst: impl Fn(&Self::Item) -> bool) -> impl Values<Item = Self::Item>
    where
        Self: Sized,
    {
        match self {
            Self::Continue(x) => match whilst(&x) {
                true => Self::Continue(x),
                false => Self::Stop,
            },
            Self::Stop => Self::Stop,
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::Continue(x) => match acc {
                Some(acc) => Some(reduce(u, acc, x)),
                None => Some(x),
            },
            Self::Stop => acc,
        }
    }

    fn first(self) -> WhilstOption<Self::Item> {
        match self {
            Self::Continue(x) => WhilstOption::ContinueSome(x),
            Self::Stop => WhilstOption::Stop,
        }
    }
}
