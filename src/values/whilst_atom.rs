use crate::values::runner_results::{
    ArbitraryPush, Fallible, Infallible, Next, OrderedPush, Reduce, SequentialPush,
};
use crate::values::whilst_atom_result::WhilstAtomResult;
use crate::values::whilst_iterators::WhilstAtomFlatMapIter;
use crate::values::{TransformableValues, Values, WhilstOption, WhilstVector};
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

    fn values_to_depracate(self) -> impl IntoIterator<Item = Self::Item> {
        match self {
            Self::Continue(x) => Some(x).into_iter(),
            _ => None.into_iter(),
        }
    }

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

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
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

    fn first_to_depracate(self) -> WhilstOption<Self::Item> {
        match self {
            Self::Continue(x) => WhilstOption::ContinueSome(x),
            Self::Stop => WhilstOption::Stop,
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
    fn map<M, O>(
        self,
        map: M,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        M: Fn(Self::Item) -> O + Clone,
    {
        match self {
            Self::Continue(x) => WhilstAtom::Continue(map(x)),
            Self::Stop => WhilstAtom::Stop,
        }
    }

    fn filter<F>(
        self,
        filter: F,
    ) -> impl TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
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

    fn flat_map<Fm, Vo>(
        self,
        flat_map: Fm,
    ) -> impl TransformableValues<Item = Vo::Item, Fallibility = Self::Fallibility>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo + Clone,
    {
        let iter = WhilstAtomFlatMapIter::from_atom(self, &flat_map);
        WhilstVector(iter)
    }

    fn filter_map<Fm, O>(
        self,
        filter_map: Fm,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
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

    fn whilst(
        self,
        whilst: impl Fn(&Self::Item) -> bool,
    ) -> impl TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
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

    fn map_while_ok<Mr, O, E>(self, map_res: Mr) -> impl Values<Item = O, Fallibility = Fallible<E>>
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

    fn u_map<U, M, O>(
        self,
        u: &mut U,
        map: M,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        M: Fn(&mut U, Self::Item) -> O,
    {
        match self {
            Self::Continue(x) => WhilstAtom::Continue(map(u, x)),
            Self::Stop => WhilstAtom::Stop,
        }
    }

    fn u_filter<U, F>(
        self,
        u: &mut U,
        filter: F,
    ) -> impl TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        F: Fn(&mut U, &Self::Item) -> bool,
    {
        match self {
            Self::Continue(x) => match filter(u, &x) {
                true => WhilstOption::ContinueSome(x),
                false => WhilstOption::ContinueNone,
            },
            Self::Stop => WhilstOption::Stop,
        }
    }

    fn u_flat_map<U, Fm, Vo>(
        self,
        u: &mut U,
        flat_map: Fm,
    ) -> impl TransformableValues<Item = Vo::Item, Fallibility = Self::Fallibility>
    where
        Vo: IntoIterator,
        Fm: Fn(&mut U, Self::Item) -> Vo,
    {
        let iter = WhilstAtomFlatMapIter::u_from_atom(u, self, &flat_map);
        WhilstVector(iter)
    }

    fn u_filter_map<U, Fm, O>(
        self,
        u: &mut U,
        filter_map: Fm,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        Fm: Fn(&mut U, Self::Item) -> Option<O>,
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
