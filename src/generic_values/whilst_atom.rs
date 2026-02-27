use crate::generic_values::runner_results::{
    ArbitraryPush, Infallible, Next, OrderedPush, Reduce, SequentialPush,
};
use crate::generic_values::whilst_atom_result::WhilstAtomResult;
use crate::generic_values::whilst_iterators::WhilstAtomIter;
use crate::generic_values::{TransformableValues, Values, WhilstOption, WhilstVector};
use alloc::vec::Vec;
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

    type Fallibility = Infallible;

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>,
    {
        match self {
            Self::Continue(x) => {
                vector.push(x);
                SequentialPush::Done
            }
            Self::Stop => SequentialPush::StoppedByWhileCondition,
        }
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility> {
        match self {
            Self::Continue(x) => {
                vec.push((idx, x));
                OrderedPush::Done
            }
            Self::Stop => OrderedPush::StoppedByWhileCondition { idx },
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> ArbitraryPush<Self::Fallibility>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self {
            Self::Continue(x) => {
                bag.push(x);
                ArbitraryPush::Done
            }
            Self::Stop => ArbitraryPush::StoppedByWhileCondition,
        }
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::Continue(x) => Reduce::Done {
                acc: Some(match acc {
                    Some(acc) => reduce(acc, x),
                    None => x,
                }),
            },
            Self::Stop => Reduce::StoppedByWhileCondition { acc },
        }
    }

    fn u_acc_reduce<U, X>(self, u: *mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(*mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::Continue(x) => Reduce::Done {
                acc: Some(match acc {
                    Some(acc) => reduce(u, acc, x),
                    None => x,
                }),
            },
            Self::Stop => Reduce::StoppedByWhileCondition { acc },
        }
    }

    fn next(self) -> Next<Self> {
        match self {
            Self::Continue(x) => Next::Done { value: Some(x) },
            Self::Stop => Next::StoppedByWhileCondition,
        }
    }
}

impl<T> TransformableValues for WhilstAtom<T> {
    type Map<M, O>
        = WhilstAtom<O>
    where
        M: Fn(Self::Item) -> O;
    fn map<M, O>(self, map: M) -> Self::Map<M, O>
    where
        M: Fn(Self::Item) -> O,
    {
        match self {
            Self::Continue(x) => WhilstAtom::Continue(map(x)),
            Self::Stop => WhilstAtom::Stop,
        }
    }

    type Filter<F>
        = WhilstOption<T>
    where
        F: Fn(&Self::Item) -> bool;
    fn filter<F>(self, filter: F) -> Self::Filter<F>
    where
        F: Fn(&Self::Item) -> bool,
    {
        match self {
            Self::Continue(x) => match filter(&x) {
                true => WhilstOption::ContinueSome(x),
                false => WhilstOption::ContinueNone,
            },
            Self::Stop => WhilstOption::Stop,
        }
    }

    type FlatMap<Fm, Vo>
        = WhilstVector<WhilstAtomIter<Vo>, Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMap<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        let atom_iter = match self {
            Self::Continue(x) => WhilstAtom::Continue(flat_map(x).into_iter()),
            Self::Stop => WhilstAtom::Stop,
        };
        let iter = WhilstAtomIter::new(atom_iter);
        WhilstVector(iter)
    }

    type FilterMap<Fm, O>
        = WhilstOption<O>
    where
        Fm: Fn(Self::Item) -> Option<O>;
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMap<Fm, O>
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

    type Whilst<W>
        = WhilstAtom<T>
    where
        W: Fn(&Self::Item) -> bool;
    fn whilst<W>(self, whilst: W) -> Self::Whilst<W>
    where
        W: Fn(&Self::Item) -> bool,
    {
        match self {
            Self::Continue(x) => match whilst(&x) {
                true => Self::Continue(x),
                false => Self::Stop,
            },
            Self::Stop => Self::Stop,
        }
    }

    type MapWhileOk<Mr, O, E>
        = WhilstAtomResult<O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send;

    fn map_while_ok<Mr, O, E>(self, map_res: Mr) -> Self::MapWhileOk<Mr, O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send,
    {
        match self {
            Self::Continue(x) => match map_res(x) {
                Ok(x) => WhilstAtomResult::ContinueOk(x),
                Err(e) => WhilstAtomResult::StopErr(e),
            },
            Self::Stop => WhilstAtomResult::StopWhile,
        }
    }

    type UMap<U, M, O>
        = WhilstAtom<O>
    where
        M: Fn(*mut U, Self::Item) -> O;
    fn u_map<U, M, O>(self, u: *mut U, map: M) -> Self::UMap<U, M, O>
    where
        M: Fn(*mut U, Self::Item) -> O,
    {
        match self {
            Self::Continue(x) => WhilstAtom::Continue(map(u, x)),
            Self::Stop => WhilstAtom::Stop,
        }
    }

    type UFilter<U, F>
        = WhilstOption<T>
    where
        F: Fn(*mut U, &Self::Item) -> bool;
    fn u_filter<U, F>(self, u: *mut U, filter: F) -> Self::UFilter<U, F>
    where
        F: Fn(*mut U, &Self::Item) -> bool,
    {
        match self {
            Self::Continue(x) => match filter(u, &x) {
                true => WhilstOption::ContinueSome(x),
                false => WhilstOption::ContinueNone,
            },
            Self::Stop => WhilstOption::Stop,
        }
    }

    type UFlatMap<U, Fm, Vo>
        = WhilstVector<WhilstAtomIter<Vo>, Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo;
    fn u_flat_map<U, Fm, Vo>(self, u: *mut U, flat_map: Fm) -> Self::UFlatMap<U, Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo,
    {
        let atom_iter = match self {
            Self::Continue(x) => WhilstAtom::Continue(flat_map(u, x).into_iter()),
            Self::Stop => WhilstAtom::Stop,
        };
        let iter = WhilstAtomIter::new(atom_iter);
        WhilstVector(iter)
    }

    type UFilterMap<U, Fm, O>
        = WhilstOption<O>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>;
    fn u_filter_map<U, Fm, O>(self, u: *mut U, filter_map: Fm) -> Self::UFilterMap<U, Fm, O>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>,
    {
        match self {
            Self::Continue(x) => match filter_map(u, x) {
                Some(x) => WhilstOption::ContinueSome(x),
                None => WhilstOption::ContinueNone,
            },
            Self::Stop => WhilstOption::Stop,
        }
    }
}
