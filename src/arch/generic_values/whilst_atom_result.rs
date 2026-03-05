use crate::generic_values::Values;
use crate::generic_values::runner_results::{
    ArbitraryPush, Fallible, Next, OrderedPush, Reduce, SequentialPush,
};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub enum WhilstAtomResult<T, E>
where
    E: Send,
{
    ContinueOk(T),
    StopErr(E),
    StopWhile,
}

impl<T, E> Values for WhilstAtomResult<T, E>
where
    E: Send,
{
    type Item = T;

    type Fallibility = Fallible<E>;

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>,
    {
        match self {
            Self::ContinueOk(x) => {
                vector.push(x);
                SequentialPush::Done
            }
            Self::StopErr(error) => SequentialPush::StoppedByError { error },
            Self::StopWhile => SequentialPush::StoppedByWhileCondition,
        }
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility> {
        match self {
            Self::ContinueOk(x) => {
                vec.push((idx, x));
                OrderedPush::Done
            }
            Self::StopErr(error) => OrderedPush::StoppedByError { idx, error },
            Self::StopWhile => OrderedPush::StoppedByWhileCondition { idx },
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> ArbitraryPush<Self::Fallibility>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self {
            Self::ContinueOk(x) => {
                bag.push(x);
                ArbitraryPush::Done
            }
            Self::StopErr(error) => ArbitraryPush::StoppedByError { error },
            Self::StopWhile => ArbitraryPush::StoppedByWhileCondition,
        }
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueOk(x) => Reduce::Done {
                acc: Some(match acc {
                    Some(acc) => reduce(acc, x),
                    None => x,
                }),
            },
            Self::StopErr(error) => Reduce::StoppedByError { error },
            Self::StopWhile => Reduce::StoppedByWhileCondition { acc },
        }
    }

    fn u_acc_reduce<U, X>(self, u: *mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(*mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueOk(x) => Reduce::Done {
                acc: Some(match acc {
                    Some(acc) => reduce(u, acc, x),
                    None => x,
                }),
            },
            Self::StopErr(error) => Reduce::StoppedByError { error },
            Self::StopWhile => Reduce::StoppedByWhileCondition { acc },
        }
    }

    fn next(self) -> Next<Self> {
        match self {
            Self::ContinueOk(x) => Next::Done { value: Some(x) },
            Self::StopErr(error) => Next::StoppedByError { error },
            Self::StopWhile => Next::StoppedByWhileCondition,
        }
    }
}
