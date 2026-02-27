use crate::generic_values::Values;
use crate::generic_values::runner_results::{
    ArbitraryPush, Infallible, Next, OrderedPush, Reduce, SequentialPush,
};
use alloc::vec::Vec;
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

    fn u_acc_reduce<U, X>(self, u: *mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(*mut U, Self::Item, Self::Item) -> Self::Item,
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
