use crate::values::{Okay, Values, WhilstOption, runner_results::ValuesPush};
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;

pub struct VectorOk<I, T, E>(pub(crate) I)
where
    I: IntoIterator<Item = Okay<T, E>>,
    E: Send;

impl<I, T, E> Values for VectorOk<I, T, E>
where
    I: IntoIterator<Item = Okay<T, E>>,
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
        for x in self.0 {
            match x.0 {
                Ok(x) => vector.push(x),
                Err(e) => return true,
            }
        }
        false
    }

    fn push_to_vec_with_idx(
        self,
        idx: usize,
        vec: &mut Vec<(usize, Self::Item)>,
    ) -> ValuesPush<Self::Error> {
        for x in self.0 {
            match x.0 {
                Ok(x) => vec.push((idx, x)),
                Err(error) => return ValuesPush::StoppedByError { idx, error },
            }
        }
        ValuesPush::Done
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> bool
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        for x in self.0 {
            match x.0 {
                Ok(x) => _ = bag.push(x),
                Err(e) => return true,
            }
        }
        false
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
                    Some(x) => match x.0 {
                        Ok(x) => x,
                        Err(e) => return (true, None), // first element is stop, acc is None
                    },
                }
            }
        };

        for x in iter {
            match x.0 {
                Ok(x) => acc = reduce(acc, x),
                Err(e) => return (true, Some(acc)),
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
                    Some(x) => match x.0 {
                        Ok(x) => x,
                        Err(e) => return None, // first element is stop, acc is None
                    },
                }
            }
        };

        for x in iter {
            match x.0 {
                Ok(x) => acc = reduce(u, acc, x),
                Err(e) => return Some(acc),
            }
        }

        Some(acc)
    }

    fn first(self) -> WhilstOption<Self::Item> {
        match self.0.into_iter().next() {
            Some(x) => match x.0 {
                Ok(x) => WhilstOption::ContinueSome(x),
                Err(e) => todo!(),
            },
            None => WhilstOption::ContinueNone,
        }
    }
}
