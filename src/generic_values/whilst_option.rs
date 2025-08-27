use crate::generic_values::runner_results::{
    ArbitraryPush, Fallible, Infallible, Next, OrderedPush, Reduce, SequentialPush,
};
use crate::generic_values::whilst_iterators::WhilstOptionFlatMapIter;
use crate::generic_values::whilst_option_result::WhilstOptionResult;
use crate::generic_values::{TransformableValues, Values, WhilstVector};
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub enum WhilstOption<T> {
    ContinueSome(T),
    ContinueNone,
    Stop,
}

impl<T> Values for WhilstOption<T> {
    type Item = T;

    type Fallibility = Infallible;

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>,
    {
        match self {
            Self::ContinueSome(x) => {
                vector.push(x);
                SequentialPush::Done
            }
            Self::ContinueNone => SequentialPush::Done,
            Self::Stop => SequentialPush::StoppedByWhileCondition,
        }
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility> {
        match self {
            Self::ContinueSome(x) => {
                vec.push((idx, x));
                OrderedPush::Done
            }
            Self::ContinueNone => OrderedPush::Done,
            Self::Stop => OrderedPush::StoppedByWhileCondition { idx },
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> ArbitraryPush<Self::Fallibility>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self {
            Self::ContinueSome(x) => {
                bag.push(x);
                ArbitraryPush::Done
            }
            Self::ContinueNone => ArbitraryPush::Done,
            Self::Stop => ArbitraryPush::StoppedByWhileCondition,
        }
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueSome(x) => Reduce::Done {
                acc: Some(match acc {
                    Some(acc) => reduce(acc, x),
                    None => x,
                }),
            },
            Self::ContinueNone => Reduce::Done { acc },
            Self::Stop => Reduce::StoppedByWhileCondition { acc },
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueSome(x) => Reduce::Done {
                acc: Some(match acc {
                    Some(acc) => reduce(u, acc, x),
                    None => x,
                }),
            },
            Self::ContinueNone => Reduce::Done { acc },
            Self::Stop => Reduce::StoppedByWhileCondition { acc },
        }
    }

    fn next(self) -> Next<Self> {
        match self {
            Self::ContinueSome(x) => Next::Done { value: Some(x) },
            Self::ContinueNone => Next::Done { value: None },
            Self::Stop => Next::StoppedByWhileCondition,
        }
    }
}

impl<T> TransformableValues for WhilstOption<T> {
    fn map<M, O>(
        self,
        map: M,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        M: Fn(Self::Item) -> O + Clone,
    {
        match self {
            Self::ContinueSome(x) => WhilstOption::ContinueSome(map(x)),
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
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
            Self::ContinueSome(x) => match filter(&x) {
                true => Self::ContinueSome(x),
                false => Self::ContinueNone,
            },
            Self::ContinueNone => Self::ContinueNone,
            Self::Stop => Self::Stop,
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
        let iter = WhilstOptionFlatMapIter::from_option(self, &flat_map);
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
            Self::ContinueSome(x) => match filter_map(x) {
                Some(x) => WhilstOption::ContinueSome(x),
                None => WhilstOption::ContinueNone,
            },
            Self::ContinueNone => WhilstOption::ContinueNone,
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
            Self::ContinueSome(x) => match whilst(&x) {
                true => Self::ContinueSome(x),
                false => Self::Stop,
            },
            Self::ContinueNone => Self::ContinueNone,
            Self::Stop => Self::Stop,
        }
    }

    fn map_while_ok<Mr, O, E>(self, map_res: Mr) -> impl Values<Item = O, Fallibility = Fallible<E>>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send,
    {
        match self {
            Self::ContinueSome(x) => match map_res(x) {
                Ok(x) => WhilstOptionResult::ContinueSomeOk(x),
                Err(e) => WhilstOptionResult::StopErr(e),
            },
            Self::ContinueNone => WhilstOptionResult::ContinueNone,
            Self::Stop => WhilstOptionResult::StopWhile,
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
            Self::ContinueSome(x) => WhilstOption::ContinueSome(map(u, x)),
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
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
            Self::ContinueSome(x) => match filter(u, &x) {
                true => Self::ContinueSome(x),
                false => Self::ContinueNone,
            },
            Self::ContinueNone => Self::ContinueNone,
            Self::Stop => Self::Stop,
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
        let iter = WhilstOptionFlatMapIter::u_from_option(u, self, &flat_map);
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
            Self::ContinueSome(x) => match filter_map(u, x) {
                Some(x) => WhilstOption::ContinueSome(x),
                None => WhilstOption::ContinueNone,
            },
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::Stop => WhilstOption::Stop,
        }
    }
}
