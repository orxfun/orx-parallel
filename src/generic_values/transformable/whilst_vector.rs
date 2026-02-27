use crate::generic_values::{
    TransformableValues, WhilstAtom, WhilstVector,
    transformable::iter::{
        WhilstVectorFilterIter, WhilstVectorFilterMapIter, WhilstVectorFlatMapIter,
        WhilstVectorMapIter, WhilstVectorUFilterIter, WhilstVectorUFilterMapIter,
        WhilstVectorUFlatMapIter, WhilstVectorUMapIter, WhilstVectorWhilstIter,
    },
    whilst_vector_result::WhilstVectorResult,
};

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

    type UFilter<U, F>
        = WhilstVector<WhilstVectorUFilterIter<U, I::IntoIter, T, F>, T>
    where
        F: Fn(*mut U, &Self::Item) -> bool;
    fn u_filter<U, F>(self, u: *mut U, filter: F) -> Self::UFilter<U, F>
    where
        F: Fn(*mut U, &Self::Item) -> bool,
    {
        let iter = WhilstVectorUFilterIter::new(u, self.0.into_iter(), filter);
        WhilstVector(iter)
    }

    type UFlatMap<U, Fm, Vo>
        = WhilstVector<WhilstVectorUFlatMapIter<U, I::IntoIter, T, Vo, Fm>, Vo::Item>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo;
    fn u_flat_map<U, Fm, Vo>(self, u: *mut U, flat_map: Fm) -> Self::UFlatMap<U, Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo,
    {
        let iter = WhilstVectorUFlatMapIter::new(u, self.0.into_iter(), flat_map);
        WhilstVector(iter)
    }

    type UFilterMap<U, Fm, O>
        = WhilstVector<WhilstVectorUFilterMapIter<U, I::IntoIter, T, O, Fm>, O>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>;
    fn u_filter_map<U, Fm, O>(self, u: *mut U, filter_map: Fm) -> Self::UFilterMap<U, Fm, O>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>,
    {
        let iter = WhilstVectorUFilterMapIter::new(u, self.0.into_iter(), filter_map);
        WhilstVector(iter)
    }
}
