use crate::values::{
    Values, WhilstOption,
    runner_results::{ArbitraryPush, OrderedPush},
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

    type Error = E;

    fn values_to_depracate(self) -> impl IntoIterator<Item = Self::Item> {
        match self {
            Ok(x) => Some(x).into_iter(),
            _ => None.into_iter(),
        }
    }

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        match self {
            Ok(x) => {
                vector.push(x);
                false
            }
            Err(e) => true,
        }
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<E> {
        match self {
            Ok(x) => {
                vec.push((idx, x));
                OrderedPush::Done
            }
            Err(error) => OrderedPush::StoppedByError { idx, error },
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> ArbitraryPush<Self::Error>
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

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Ok(x) => match acc {
                Some(acc) => (false, Some(reduce(acc, x))),
                None => (false, Some(x)),
            },
            Err(e) => (true, None), // resets entire reduction so far!
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Ok(x) => match acc {
                Some(acc) => Some(reduce(u, acc, x)),
                None => Some(x),
            },
            Err(e) => None, // resets entire reduction so far!
        }
    }

    fn first(self) -> WhilstOption<Self::Item> {
        todo!()
    }
}
