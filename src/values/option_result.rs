use crate::values::{Values, runner_results::OrderedPush, whilst_option::WhilstOption};
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub struct OptionResult<T, E>(pub(super) Option<Result<T, E>>)
where
    E: Send;

impl<T, E> Values for OptionResult<T, E>
where
    E: Send,
{
    type Item = T;

    type Error = E;

    fn values_to_depracate(self) -> impl IntoIterator<Item = Self::Item> {
        todo!();
        core::iter::empty()
    }

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        match self.0 {
            Some(Ok(x)) => {
                vector.push(x);
                false
            }
            Some(Err(error)) => true,
            None => false,
        }
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Error> {
        match self.0 {
            Some(Ok(x)) => {
                vec.push((idx, x));
                OrderedPush::Done
            }
            Some(Err(error)) => OrderedPush::StoppedByError { idx, error },
            None => OrderedPush::Done,
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> bool
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self.0 {
            Some(Ok(x)) => {
                _ = bag.push(x);
                false
            }
            Some(Err(error)) => true,
            None => false,
        }
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self.0 {
            Some(Ok(x)) => {
                let acc = Some(match acc {
                    Some(y) => reduce(y, x),
                    None => x,
                });
                (false, acc)
            }
            Some(Err(error)) => (true, None),
            None => (false, acc),
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self.0 {
            Some(Ok(x)) => {
                let acc = Some(match acc {
                    Some(y) => reduce(u, y, x),
                    None => x,
                });
                acc
            }
            Some(Err(error)) => None,
            None => acc,
        }
    }

    fn first(self) -> WhilstOption<Self::Item> {
        match self.0 {
            Some(Ok(x)) => WhilstOption::ContinueSome(x),
            Some(Err(error)) => WhilstOption::Stop,
            None => WhilstOption::ContinueNone,
        }
    }
}
