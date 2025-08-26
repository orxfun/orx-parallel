use crate::values::Values;
use crate::values::runner_results::{
    ArbitraryPush, Fallible, Next, OrderedPush, Reduce, SequentialPush,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

/// Represents scalar value for early stopping error cases:
///
/// * Whenever computation creates an error at any point, all computed values are irrelevant,
///   the only relevant value is the created error.
/// * Computed values are relevant iff entire inputs result in an Ok variant.
/// * Therefore, observation of an error case allows to immediately stop computation.

impl<T, E> Values for Result<T, E>
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
            Ok(x) => {
                vector.push(x);
                SequentialPush::Done
            }
            Err(error) => SequentialPush::StoppedByError { error },
        }
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility> {
        match self {
            Ok(x) => {
                vec.push((idx, x));
                OrderedPush::Done
            }
            Err(error) => OrderedPush::StoppedByError { idx, error },
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> ArbitraryPush<Self::Fallibility>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self {
            Ok(x) => {
                bag.push(x);
                ArbitraryPush::Done
            }
            Err(error) => ArbitraryPush::StoppedByError { error },
        }
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Ok(x) => Reduce::Done {
                acc: Some(match acc {
                    Some(acc) => reduce(acc, x),
                    None => x,
                }),
            },
            Err(error) => Reduce::StoppedByError { error },
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Ok(x) => Reduce::Done {
                acc: Some(match acc {
                    Some(acc) => reduce(u, acc, x),
                    None => x,
                }),
            },
            Err(error) => Reduce::StoppedByError { error },
        }
    }

    fn next(self) -> Next<Self> {
        match self {
            Ok(x) => Next::Done { value: Some(x) },
            Err(error) => Next::StoppedByError { error },
        }
    }
}
