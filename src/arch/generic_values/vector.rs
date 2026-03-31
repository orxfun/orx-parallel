use crate::generic_values::{
    Values,
    runner_results::{ArbitraryPush, Infallible, Next, OrderedPush, Reduce, SequentialPush},
};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;

pub struct Vector<I>(pub I)
where
    I: IntoIterator;

impl<I> Values for Vector<I>
where
    I: IntoIterator,
{
    type Item = I::Item;

    type Fallibility = Infallible;

    #[inline(always)]
    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            vector.push(x);
        }
        SequentialPush::Done
    }

    #[inline(always)]
    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility> {
        for x in self.0 {
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
        for x in self.0 {
            bag.push(x);
        }
        ArbitraryPush::Done
    }

    #[inline(always)]
    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        let reduced = self.0.into_iter().reduce(&reduce);

        Reduce::Done {
            acc: match (acc, reduced) {
                (Some(x), Some(y)) => Some(reduce(x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            },
        }
    }

    fn u_acc_reduce<U, X>(self, u: *mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(*mut U, Self::Item, Self::Item) -> Self::Item,
    {
        let reduced = self.0.into_iter().reduce(|a, b| reduce(u, a, b));

        Reduce::Done {
            acc: match (acc, reduced) {
                (Some(x), Some(y)) => Some(reduce(u, x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            },
        }
    }

    fn next(self) -> Next<Self> {
        Next::Done {
            value: self.0.into_iter().next(),
        }
    }
}
