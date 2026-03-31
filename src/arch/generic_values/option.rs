use crate::generic_values::Values;
use crate::generic_values::runner_results::{
    ArbitraryPush, Infallible, Next, OrderedPush, Reduce, SequentialPush,
};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

impl<T> Values for Option<T> {
    type Item = T;

    type Fallibility = Infallible;

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>,
    {
        if let Some(x) = self {
            vector.push(x)
        }
        SequentialPush::Done
    }

    #[inline(always)]
    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility> {
        if let Some(x) = self {
            vec.push((idx, x));
        }
        OrderedPush::Done
    }

    #[inline(always)]
    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> ArbitraryPush<Self::Fallibility>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        if let Some(x) = self {
            bag.push(x);
        }
        ArbitraryPush::Done
    }

    #[inline(always)]
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        Reduce::Done {
            acc: match (acc, self) {
                (Some(x), Some(y)) => Some(reduce(x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            },
        }
    }

    #[inline(always)]
    fn u_acc_reduce<U, X>(self, u: *mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(*mut U, Self::Item, Self::Item) -> Self::Item,
    {
        Reduce::Done {
            acc: match (acc, self) {
                (Some(x), Some(y)) => Some(reduce(u, x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            },
        }
    }

    fn next(self) -> Next<Self> {
        Next::Done { value: self }
    }
}
