use crate::generic_values::Values;
use crate::generic_values::runner_results::{
    ArbitraryPush, Fallible, Next, OrderedPush, Reduce, SequentialPush,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub struct OptionResult<T, E>(pub(crate) Option<Result<T, E>>)
where
    E: Send;

impl<T, E> Values for OptionResult<T, E>
where
    E: Send,
{
    type Item = T;

    type Fallibility = Fallible<E>;

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>,
    {
        match self.0 {
            Some(Ok(x)) => {
                vector.push(x);
                SequentialPush::Done
            }
            Some(Err(error)) => SequentialPush::StoppedByError { error },
            None => SequentialPush::Done,
        }
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility> {
        match self.0 {
            Some(Ok(x)) => {
                vec.push((idx, x));
                OrderedPush::Done
            }
            Some(Err(error)) => OrderedPush::StoppedByError { idx, error },
            None => OrderedPush::Done,
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> ArbitraryPush<Self::Fallibility>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self.0 {
            Some(Ok(x)) => {
                _ = bag.push(x);
                ArbitraryPush::Done
            }
            Some(Err(error)) => ArbitraryPush::StoppedByError { error },
            None => ArbitraryPush::Done,
        }
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self.0 {
            Some(Ok(x)) => Reduce::Done {
                acc: Some(match acc {
                    Some(y) => reduce(y, x),
                    None => x,
                }),
            },
            None => Reduce::Done { acc },
            Some(Err(error)) => Reduce::StoppedByError { error },
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self.0 {
            Some(Ok(x)) => Reduce::Done {
                acc: Some(match acc {
                    Some(y) => reduce(u, y, x),
                    None => x,
                }),
            },
            None => Reduce::Done { acc },
            Some(Err(error)) => Reduce::StoppedByError { error },
        }
    }

    fn next(self) -> Next<Self> {
        match self.0 {
            Some(Ok(value)) => Next::Done { value: Some(value) },
            None => Next::Done { value: None },
            Some(Err(error)) => Next::StoppedByError { error },
        }
    }
}
