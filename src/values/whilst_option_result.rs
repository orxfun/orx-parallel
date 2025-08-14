use crate::values::runner_results::ValuesPush;
use crate::values::whilst_iterators::WhilstOptionFlatMapIter;
use crate::values::{TransformableValues, Values, WhilstOption, WhilstVector};
use orx_concurrent_bag::ConcurrentBag;
use orx_pinned_vec::{IntoConcurrentPinnedVec, PinnedVec};

pub enum WhilstOptionResult<T, E>
where
    E: Send,
{
    ContinueSomeOk(T),
    ContinueNone,
    StopErr(E),
    StopWhile,
}

impl<T, E> Values for WhilstOptionResult<T, E>
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
            Self::ContinueSomeOk(x) => {
                vector.push(x);
                false
            }
            Self::ContinueNone => false,
            Self::StopErr(error) => true,
            Self::StopWhile => true,
        }
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> ValuesPush<Self::Error> {
        match self {
            Self::ContinueSomeOk(x) => {
                vec.push((idx, x));
                ValuesPush::Done
            }
            Self::ContinueNone => ValuesPush::Done,
            Self::StopErr(error) => ValuesPush::StoppedByError { idx, error },
            Self::StopWhile => ValuesPush::StoppedByWhileCondition { idx },
        }
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> bool
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        match self {
            Self::ContinueSomeOk(x) => {
                bag.push(x);
                false
            }
            Self::ContinueNone => false,
            Self::StopErr(error) => true,
            Self::StopWhile => true,
        }
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueSomeOk(x) => {
                let acc = Some(match acc {
                    Some(y) => reduce(y, x),
                    None => x,
                });
                (false, acc)
            }
            Self::ContinueNone => (false, acc),
            Self::StopErr(error) => (true, None),
            Self::StopWhile => (true, acc),
        }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        match self {
            Self::ContinueSomeOk(x) => {
                let acc = Some(match acc {
                    Some(y) => reduce(u, y, x),
                    None => x,
                });
                acc
            }
            Self::ContinueNone => acc,
            Self::StopErr(error) => None,
            Self::StopWhile => acc,
        }
    }

    fn first(self) -> WhilstOption<Self::Item> {
        match self {
            Self::ContinueSomeOk(x) => WhilstOption::ContinueSome(x),
            Self::ContinueNone => WhilstOption::ContinueNone,
            Self::StopErr(error) => WhilstOption::Stop,
            Self::StopWhile => WhilstOption::Stop,
        }
    }
}
