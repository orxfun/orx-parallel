use super::transformable_values::TransformableValues;
use crate::generic_values::{
    Values, WhilstAtom,
    runner_results::{
        ArbitraryPush, Fallible, Infallible, Next, OrderedPush, Reduce, SequentialPush,
    },
    whilst_iterators::{
        WhilstAtomFlatMapIter, WhilstVectorFilterIter, WhilstVectorFilterMapIter,
        WhilstVectorFlatMapIter, WhilstVectorMapIter, WhilstVectorUMapIter, WhilstVectorWhilstIter,
    },
    whilst_vector_result::WhilstVectorResult,
};
use alloc::vec::Vec;
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

    type Fallibility = Infallible;

    fn push_to_pinned_vec<P>(self, vector: &mut P) -> SequentialPush<Self::Fallibility>
    where
        P: PinnedVec<Self::Item>,
    {
        for x in self.0 {
            match x {
                WhilstAtom::Continue(x) => vector.push(x),
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
                WhilstAtom::Continue(x) => vec.push((idx, x)),
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
                WhilstAtom::Continue(x) => _ = bag.push(x),
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
                    Some(x) => match x {
                        WhilstAtom::Continue(x) => x,
                        WhilstAtom::Stop => return Reduce::StoppedByWhileCondition { acc: None }, // first element is stop, acc is None
                    },
                }
            }
        };

        for x in iter {
            match x {
                WhilstAtom::Continue(x) => acc = reduce(acc, x),
                WhilstAtom::Stop => return Reduce::StoppedByWhileCondition { acc: Some(acc) },
            }
        }

        Reduce::Done { acc: Some(acc) }
    }

    fn u_acc_reduce<U, X>(self, u: *mut U, acc: Option<Self::Item>, reduce: X) -> Reduce<Self>
    where
        X: Fn(*mut U, Self::Item, Self::Item) -> Self::Item,
    {
        let mut iter = self.0.into_iter();

        let mut acc = match acc {
            Some(x) => x,
            None => {
                let first = iter.next();
                match first {
                    None => return Reduce::Done { acc: None }, // empty iterator but not stopped, acc is None
                    Some(x) => match x {
                        WhilstAtom::Continue(x) => x,
                        WhilstAtom::Stop => return Reduce::StoppedByWhileCondition { acc: None }, // first element is stop, acc is None
                    },
                }
            }
        };

        for x in iter {
            match x {
                WhilstAtom::Continue(x) => acc = reduce(u, acc, x),
                WhilstAtom::Stop => return Reduce::StoppedByWhileCondition { acc: Some(acc) },
            }
        }

        Reduce::Done { acc: Some(acc) }
    }

    fn next(self) -> Next<Self> {
        match self.0.into_iter().next() {
            Some(x) => match x {
                WhilstAtom::Continue(x) => Next::Done { value: Some(x) },
                WhilstAtom::Stop => Next::StoppedByWhileCondition,
            },
            None => Next::Done { value: None },
        }
    }
}

impl<I, T> TransformableValues for WhilstVector<I, T>
where
    I: IntoIterator<Item = WhilstAtom<T>>,
{
    type Map<M, O>
        = WhilstVector<WhilstVectorMapIter<I::IntoIter, T, O, M>, O>
    where
        M: Fn(Self::Item) -> O;
    fn map<M, O>(self, map: M) -> Self::Map<M, O>
    where
        M: Fn(Self::Item) -> O,
    {
        let iter = WhilstVectorMapIter::new(self.0.into_iter(), map);
        WhilstVector(iter)
    }

    type Filter<F>
        = WhilstVector<WhilstVectorFilterIter<I::IntoIter, T, F>, T>
    where
        F: Fn(&Self::Item) -> bool;
    fn filter<F>(self, filter: F) -> Self::Filter<F>
    where
        F: Fn(&Self::Item) -> bool,
    {
        let iter = WhilstVectorFilterIter::new(self.0.into_iter(), filter);
        WhilstVector(iter)
    }

    type FlatMap<Fm, Vo>
        = WhilstVector<WhilstVectorFlatMapIter<I::IntoIter, T, Vo, Fm>, Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMap<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        let iter = WhilstVectorFlatMapIter::new(self.0.into_iter(), flat_map);
        WhilstVector(iter)
    }

    type FilterMap<Fm, O>
        = WhilstVector<WhilstVectorFilterMapIter<I::IntoIter, T, O, Fm>, O>
    where
        Fm: Fn(Self::Item) -> Option<O>;
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMap<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        let iter = WhilstVectorFilterMapIter::new(self.0.into_iter(), filter_map);
        WhilstVector(iter)
    }

    type Whilst<W>
        = WhilstVector<WhilstVectorWhilstIter<I::IntoIter, T, W>, T>
    where
        W: Fn(&Self::Item) -> bool;
    fn whilst<W>(self, whilst: W) -> Self::Whilst<W>
    where
        W: Fn(&Self::Item) -> bool,
    {
        let iter = WhilstVectorWhilstIter::new(self.0.into_iter(), whilst);
        WhilstVector(iter)
    }

    type MapWhileOk<Mr, O, E>
        = WhilstVectorResult<WhilstVectorMapIter<I::IntoIter, T, Result<O, E>, Mr>, O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send;
    fn map_while_ok<Mr, O, E>(self, map_res: Mr) -> Self::MapWhileOk<Mr, O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send,
    {
        let iter = WhilstVectorMapIter::new(self.0.into_iter(), map_res);
        WhilstVectorResult(iter)
    }

    type UMap<U, M, O>
        = WhilstVector<WhilstVectorUMapIter<U, I::IntoIter, T, O, M>, O>
    where
        M: Fn(*mut U, Self::Item) -> O;
    fn u_map<U, M, O>(self, u: *mut U, map: M) -> Self::UMap<U, M, O>
    where
        M: Fn(*mut U, Self::Item) -> O,
    {
        let iter = WhilstVectorUMapIter::new(u, self.0.into_iter(), map);
        WhilstVector(iter)
    }

    fn u_filter<U, F>(
        self,
        u: *mut U,
        filter: F,
    ) -> impl TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        F: Fn(*mut U, &Self::Item) -> bool,
    {
        let iter = self.0.into_iter().filter_map(move |x| match x {
            WhilstAtom::Continue(x) => match filter(u, &x) {
                true => Some(WhilstAtom::Continue(x)),
                false => None,
            },
            WhilstAtom::Stop => Some(WhilstAtom::Stop),
        });
        WhilstVector(iter)
    }

    fn u_flat_map<U, Fm, Vo>(
        self,
        u: *mut U,
        flat_map: Fm,
    ) -> impl TransformableValues<Item = Vo::Item, Fallibility = Self::Fallibility>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo,
    {
        let iter = self
            .0
            .into_iter()
            .flat_map(move |atom| WhilstAtomFlatMapIter::u_from_atom(u, atom, &flat_map));
        WhilstVector(iter)
    }

    fn u_filter_map<U, Fm, O>(
        self,
        u: *mut U,
        filter_map: Fm,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>,
    {
        let iter = self.0.into_iter().filter_map(move |x| match x {
            WhilstAtom::Continue(x) => filter_map(u, x).map(WhilstAtom::Continue),
            WhilstAtom::Stop => Some(WhilstAtom::Stop),
        });
        WhilstVector(iter)
    }
}
