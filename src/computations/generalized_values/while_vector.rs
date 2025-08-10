use super::values::Values;
use orx_concurrent_bag::ConcurrentBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;
use orx_pinned_vec::PinnedVec;
use std::marker::PhantomData;

pub struct WhileVector<I, T>(I, PhantomData<T>)
where
    I: IntoIterator<Item = Option<T>>;

impl<I, T> WhileVector<I, T>
where
    I: IntoIterator<Item = Option<T>>,
{
    pub fn new(iter: I) -> Self {
        Self(iter, PhantomData)
    }
}

impl<I, T> Values for WhileVector<I, T>
where
    I: IntoIterator<Item = Option<T>>,
{
    type Item = T;

    fn values(self) -> impl IntoIterator<Item = Self::Item> {
        self.0
            .into_iter()
            .take_while(|x| x.is_some())
            .map(|x| x.unwrap()) // SAFETY: taken while x is Some
    }

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> bool
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            match x {
                Some(x) => vector.push(x),
                None => return true,
            }
        }
        false
    }

    fn push_to_vec_with_idx(self, idx: usize, vec: &mut Vec<(usize, Self::Item)>) -> Option<usize> {
        for x in self.0 {
            match x {
                Some(x) => vec.push((idx, x)),
                None => return Some(idx),
            }
        }
        None
    }

    fn push_to_bag<P>(self, bag: &ConcurrentBag<Self::Item, P>) -> Option<usize>
    where
        P: IntoConcurrentPinnedVec<Self::Item>,
        Self::Item: Send,
    {
        todo!()
    }

    fn map<M, O>(self, map: M) -> impl Values<Item = O>
    where
        M: Fn(Self::Item) -> O,
    {
        todo!()
    }

    #[inline(always)]
    fn filter<F>(self, filter: F) -> impl Values<Item = Self::Item>
    where
        F: Fn(&Self::Item) -> bool,
    {
        todo!()
    }

    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> impl Values<Item = Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        todo!()
    }

    fn filter_map<Fm, O>(self, filter_map: Fm) -> impl Values<Item = O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        todo!()
    }

    fn acc_reduce<X>(self, acc: Option<Self::Item>, reduce: X) -> (bool, Option<Self::Item>)
    where
        X: Fn(Self::Item, Self::Item) -> Self::Item,
    {
        todo!()
    }

    fn u_acc_reduce<U, X>(self, u: &mut U, acc: Option<Self::Item>, reduce: X) -> Option<Self::Item>
    where
        X: Fn(&mut U, Self::Item, Self::Item) -> Self::Item,
    {
        todo!()
    }

    fn first(self) -> Option<Self::Item> {
        todo!()
    }
}
