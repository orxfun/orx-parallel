use super::values::Values;
use crate::computations::generalized_values::while_iterators::{
    WhileIterFilter, WhileIterFilterMap, WhileIterFlatMap, WhileIterMap, WhileNext,
};
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;
use std::marker::PhantomData;

pub struct WhileVector<I, T>(I, PhantomData<T>)
where
    I: IntoIterator<Item = WhileNext<T>>;

impl<I, T> WhileVector<I, T>
where
    I: IntoIterator<Item = WhileNext<T>>,
{
    pub fn new(iter: I) -> Self {
        Self(iter, PhantomData)
    }
}

impl<I, T> Values for WhileVector<I, T>
where
    I: IntoIterator<Item = WhileNext<T>>,
{
    type Item = T;

    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        // self.0
        //     .into_iter()
        //     .take_while(|x| x.is_some())
        //     .map(|x| x.unwrap()) // SAFETY: taken while x is Some
        // todo!()
        core::iter::empty()
    }

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            match x.into_continue() {
                Some(x) => vector.push(x),
                None => return true,
            }
        }
        false
    }

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) -> Option<usize> {
        for x in self.0 {
            match x.into_continue() {
                Some(x) => vec.push((idx, x)),
                None => return Some(idx),
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
            match x.into_continue() {
                Some(x) => _ = bag.push(x),
                None => return true,
            }
        }
        false
    }

    fn map<M, O>(self, map: M) -> impl Values<Item = O>
    where
        M: Fn(Self::Item) -> O,
    {
        let iter = WhileIterMap::new(self.0.into_iter(), map);
        WhileVector::new(iter)
    }

    #[inline(always)]
    fn filter<F>(self, filter: F) -> impl Values<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool,
    {
        let iter = WhileIterFilter::new(self.0.into_iter(), filter);
        WhileVector::new(iter)
    }

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> impl Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        let iter = WhileIterFlatMap::new(self.0.into_iter(), flat_map);
        WhileVector::new(iter)
    }

    fn filter_map<Fm, O>(self, filter_map: Fm) -> impl Values<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        let iter = WhileIterFilterMap::new(self.0.into_iter(), filter_map);
        WhileVector::new(iter)
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
                    None => return (false, None), // empty iterator -> not stopped
                    Some(x) => match x.into_continue() {
                        Some(x) => x,
                        None => return (true, None), // first element is stop
                    },
                }
            }
        };

        for x in iter {
            match x.into_continue() {
                Some(x) => acc = reduce(acc, x),
                None => return (true, Some(acc)),
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
                    None => return None, // empty iterator -> not stopped
                    Some(x) => match x.into_continue() {
                        Some(x) => x,
                        None => return None, // first element is stop
                    },
                }
            }
        };

        for x in iter {
            match x.into_continue() {
                Some(x) => acc = reduce(u, acc, x),
                None => return Some(acc),
            }
        }

        Some(acc)
    }

    fn first(self) -> Option<Self::Item> {
        self.0.into_iter().next().and_then(|x| x.into_continue())
    }
}
