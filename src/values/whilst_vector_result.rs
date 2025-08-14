use crate::values::{
    Values, WhilstAtom, WhilstOption,
    runner_results::{ArbitraryPush, Fallible, OrderedPush},
};
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;

pub struct WhilstVectorResult<I, T, E>(pub(crate) I)
where
    I: IntoIterator<Item = WhilstAtom<Result<T, E>>>,
    E: Send;

impl<I, T, E> Values for WhilstVectorResult<I, T, E>
where
    I: IntoIterator<Item = WhilstAtom<Result<T, E>>>,
    E: Send,
{
    type Item = T;

    type Fallibility = Fallible<E>;

    fn values_to_depracate(self) -> impl IntoIterator<Item = Self::Item> {
        todo!();
        core::iter::empty()
    }

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            match x {
                WhilstAtom::Continue(Ok(x)) => vector.push(x),
                WhilstAtom::Continue(Err(error)) => return true,
                WhilstAtom::Stop => return true,
            }
        }
        false
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> OrderedPush<Self::Fallibility> {
        for x in self.0 {
            match x {
                WhilstAtom::Continue(Ok(x)) => vec.push((idx, x)),
                WhilstAtom::Continue(Err(error)) => {
                    return OrderedPush::StoppedByError { idx, error };
                }
                WhilstAtom::Stop => return OrderedPush::StoppedByWhileCondition { idx },
            }
        }
        OrderedPush::Done
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> ArbitraryPush<Self::Fallibility>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        for x in self.0 {
            match x {
                WhilstAtom::Continue(Ok(x)) => _ = bag.push(x),
                WhilstAtom::Continue(Err(error)) => return ArbitraryPush::StoppedByError { error },
                WhilstAtom::Stop => return ArbitraryPush::StoppedByWhileCondition,
            }
        }
        ArbitraryPush::Done
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        let mut iter = self.0.into_iter();

        let mut acc = match acc {
            Some(x) => x,
            None => {
                let first = iter.next();
                match first {
                    None => return (false, None), // empty iterator but not stopped, acc is None
                    Some(first) => match first {
                        WhilstAtom::Continue(Ok(x)) => x,
                        WhilstAtom::Continue(Err(error)) => return (true, None),
                        WhilstAtom::Stop => return (true, None), // first element is stop, acc is None
                    },
                }
            }
        };

        for x in iter {
            match x {
                WhilstAtom::Continue(Ok(x)) => acc = reduce(acc, x),
                WhilstAtom::Continue(Err(error)) => return (true, None),
                WhilstAtom::Stop => return (true, Some(acc)),
            }
        }

        (false, Some(acc))
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        let mut iter = self.0.into_iter();

        let mut acc = match acc {
            Some(x) => x,
            None => {
                let first = iter.next();
                match first {
                    None => return None, // empty iterator but not stopped, acc is None
                    Some(first) => match first {
                        WhilstAtom::Continue(Ok(x)) => x,
                        WhilstAtom::Continue(Err(error)) => return None,
                        WhilstAtom::Stop => return None, // first element is stop, acc is None
                    },
                }
            }
        };

        for x in iter {
            match x {
                WhilstAtom::Continue(Ok(x)) => acc = reduce(u, acc, x),
                WhilstAtom::Continue(Err(error)) => return None,
                WhilstAtom::Stop => return Some(acc),
            }
        }

        Some(acc)
    }

    fn first(self) -> WhilstOption<Self::Item> {
        match self.0.into_iter().next() {
            Some(x) => match x {
                WhilstAtom::Continue(Ok(x)) => WhilstOption::ContinueSome(x),
                WhilstAtom::Continue(Err(error)) => WhilstOption::Stop,
                WhilstAtom::Stop => WhilstOption::Stop,
            },
            None => WhilstOption::ContinueNone,
        }
    }
}
