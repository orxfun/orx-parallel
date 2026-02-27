use crate::generic_values::Values;
use crate::generic_values::runner_results::{
    ArbitraryPush, Infallible, Next, OrderedPush, Reduce, SequentialPush,
};
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
