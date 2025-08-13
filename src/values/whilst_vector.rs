use super::values::Values;
use crate::values::{
    whilst_atom::WhilstAtom, whilst_iterators::WhilstAtomFlatMapIter, whilst_option::WhilstOption,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;

pub struct WhilstVector<I, T>(pub(crate) I)
where
    I: IntoIterator<Item = WhilstAtom<T>>;

impl<I, T> Values for WhilstVector<I, T>
where
    I: IntoIterator<Item = WhilstAtom<T>>,
{
    type Item = T;

    type Error = ();

    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        todo!();
        core::iter::empty()
    }

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            match x {
                WhilstAtom::Continue(x) => vector.push(x),
                WhilstAtom::Stop => return true,
            }
        }
        false
    }

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) -> Option<usize> {
        for x in self.0 {
            match x {
                WhilstAtom::Continue(x) => vec.push((idx, x)),
                WhilstAtom::Stop => return Some(idx),
            }
        }
        None
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> bool
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        for x in self.0 {
            match x {
                WhilstAtom::Continue(x) => _ = bag.push(x),
                WhilstAtom::Stop => return true,
            }
        }
        false
    }

    fn map<M, O>(self, map: M) -> impl Values<Item = O>
    where
        M: Fn(Self::Item) -> O,
    {
        let iter = self.0.into_iter().map(move |x| match x {
            WhilstAtom::Continue(x) => WhilstAtom::Continue(map(x)),
            WhilstAtom::Stop => WhilstAtom::Stop,
        });
        WhilstVector(iter)
    }

    fn filter<F>(self, filter: F) -> impl Values<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Clone,
    {
        let iter = self.0.into_iter().filter_map(move |x| match x {
            WhilstAtom::Continue(x) => match filter(&x) {
                true => Some(WhilstAtom::Continue(x)),
                false => None,
            },
            WhilstAtom::Stop => Some(WhilstAtom::Stop),
        });
        WhilstVector(iter)
    }

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> impl Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        let iter = self
            .0
            .into_iter()
            .flat_map(move |atom| WhilstAtomFlatMapIter::from_atom(atom, &flat_map));
        WhilstVector(iter)
    }

    fn filter_map<Fm, O>(self, filter_map: Fm) -> impl Values<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        let iter = self.0.into_iter().filter_map(move |x| match x {
            WhilstAtom::Continue(x) => match filter_map(x) {
                Some(x) => Some(WhilstAtom::Continue(x)),
                None => None,
            },
            WhilstAtom::Stop => Some(WhilstAtom::Stop),
        });
        WhilstVector(iter)
    }

    fn whilst(self, whilst: impl Fn(&Self::Item) -> bool) -> impl Values<Item = Self::Item>
    where
        Self: Sized,
    {
        let iter = self.0.into_iter().map(move |x| match x {
            WhilstAtom::Continue(x) => match whilst(&x) {
                true => WhilstAtom::Continue(x),
                false => WhilstAtom::Stop,
            },
            WhilstAtom::Stop => WhilstAtom::Stop,
        });
        WhilstVector(iter)
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
                    Some(x) => match x {
                        WhilstAtom::Continue(x) => x,
                        WhilstAtom::Stop => return (true, None), // first element is stop, acc is None
                    },
                }
            }
        };

        for x in iter {
            match x {
                WhilstAtom::Continue(x) => acc = reduce(acc, x),
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
                    Some(x) => match x {
                        WhilstAtom::Continue(x) => x,
                        WhilstAtom::Stop => return None, // first element is stop, acc is None
                    },
                }
            }
        };

        for x in iter {
            match x {
                WhilstAtom::Continue(x) => acc = reduce(u, acc, x),
                WhilstAtom::Stop => return Some(acc),
            }
        }

        Some(acc)
    }

    fn first(self) -> WhilstOption<Self::Item> {
        match self.0.into_iter().next() {
            Some(x) => match x {
                WhilstAtom::Continue(x) => WhilstOption::ContinueSome(x),
                WhilstAtom::Stop => WhilstOption::Stop,
            },
            None => WhilstOption::ContinueNone,
        }
    }
}
