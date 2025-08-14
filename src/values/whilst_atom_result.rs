use crate::values::runner_results::OrderedPush;
use crate::values::{Values, WhilstOption};
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

    type Error = E;

    fn values_to_depracate(self) -> impl IntoIterator<Item = Self::Item> {
        todo!();
        core::iter::empty()
    }

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        match self {
            Self::ContinueOk(x) => {
                vector.push(x);
                false
            }
            Self::StopErr(error) => true,
            Self::StopWhile => true,
        }
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Error> {
        match self {
            Self::ContinueOk(x) => {
                vec.push((idx, x));
                OrderedPush::Done
            }
            Self::StopErr(error) => OrderedPush::StoppedByError { idx, error },
            Self::StopWhile => OrderedPush::StoppedByWhileCondition { idx },
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> bool
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self {
            Self::ContinueOk(x) => {
                bag.push(x);
                false
            }
            Self::StopErr(error) => true,
            Self::StopWhile => true,
        }
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueOk(x) => match acc {
                Some(acc) => (false, Some(reduce(acc, x))),
                None => (false, Some(x)),
            },
            Self::StopErr(error) => (true, None),
            Self::StopWhile => (true, acc),
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueOk(x) => match acc {
                Some(acc) => Some(reduce(u, acc, x)),
                None => Some(x),
            },
            Self::StopErr(error) => None,
            Self::StopWhile => acc,
        }
    }

    fn first(self) -> WhilstOption<Self::Item> {
        match self {
            Self::ContinueOk(x) => WhilstOption::ContinueSome(x),
            Self::StopErr(error) => WhilstOption::Stop,
            Self::StopWhile => WhilstOption::Stop,
        }
    }
}
