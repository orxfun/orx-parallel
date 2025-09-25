use crate::generic_values::{
    Values, WhilstAtom,
    runner_results::{ArbitraryPush, Fallible, Next, OrderedPush, Reduce, SequentialPush},
};
use alloc::vec::Vec;
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

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            match x {
                WhilstAtom::Continue(Ok(x)) => vector.push(x),
                WhilstAtom::Continue(Err(error)) => {
                    return SequentialPush::StoppedByError { error };
                }
                WhilstAtom::Stop => return SequentialPush::StoppedByWhileCondition,
            }
        }
        SequentialPush::Done
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

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        let mut iter = self.0.into_iter();

        let mut acc = match acc {
            Some(x) => x,
            None => {
                let first = iter.next();
                match first {
                    None => return Reduce::Done { acc: None }, // empty iterator but not stopped, acc is None
                    Some(first) => match first {
                        WhilstAtom::Continue(Ok(x)) => x,
                        WhilstAtom::Continue(Err(error)) => {
                            return Reduce::StoppedByError { error };
                        }
                        WhilstAtom::Stop => return Reduce::StoppedByWhileCondition { acc: None }, // first element is stop, acc is None
                    },
                }
            }
        };

        for x in iter {
            match x {
                WhilstAtom::Continue(Ok(x)) => acc = reduce(acc, x),
                WhilstAtom::Continue(Err(error)) => return Reduce::StoppedByError { error },
                WhilstAtom::Stop => return Reduce::StoppedByWhileCondition { acc: Some(acc) },
            }
        }

        Reduce::Done { acc: Some(acc) }
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        let mut iter = self.0.into_iter();

        let mut acc = match acc {
            Some(x) => x,
            None => {
                let first = iter.next();
                match first {
                    None => return Reduce::Done { acc: None }, // empty iterator but not stopped, acc is None
                    Some(first) => match first {
                        WhilstAtom::Continue(Ok(x)) => x,
                        WhilstAtom::Continue(Err(error)) => {
                            return Reduce::StoppedByError { error };
                        }
                        WhilstAtom::Stop => return Reduce::StoppedByWhileCondition { acc: None }, // first element is stop, acc is None
                    },
                }
            }
        };

        for x in iter {
            match x {
                WhilstAtom::Continue(Ok(x)) => acc = reduce(u, acc, x),
                WhilstAtom::Continue(Err(error)) => return Reduce::StoppedByError { error },
                WhilstAtom::Stop => return Reduce::StoppedByWhileCondition { acc: Some(acc) },
            }
        }

        Reduce::Done { acc: Some(acc) }
    }

    fn next(self) -> Next<Self> {
        match self.0.into_iter().next() {
            Some(x) => match x {
                WhilstAtom::Continue(Ok(x)) => Next::Done { value: Some(x) },
                WhilstAtom::Continue(Err(error)) => Next::StoppedByError { error },
                WhilstAtom::Stop => Next::StoppedByWhileCondition,
            },
            None => Next::Done { value: None },
        }
    }
}
