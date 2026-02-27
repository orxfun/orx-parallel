use crate::generic_values::{
    TransformableValues, Vector, VectorResult, WhilstVector,
    whilst_iterators::{
        VectorUFilterIter, VectorUFilterMapIter, VectorUFlatMapIter, VectorUMapIter,
        VectorWhilstIter,
    },
};

impl<I> TransformableValues for Vector<I>
where
    I: IntoIterator,
{
    type Map<M, O>
        = Vector<core::iter::Map<I::IntoIter, M>>
    where
        M: Fn(Self::Item) -> O;
    #[inline(always)]
    fn map<M, O>(self, map: M) -> Self::Map<M, O>
    where
        M: Fn(Self::Item) -> O,
    {
        Vector(self.0.into_iter().map(map))
    }

    type Filter<F>
        = Vector<core::iter::Filter<I::IntoIter, F>>
    where
        F: Fn(&Self::Item) -> bool;
    #[inline(always)]
    fn filter<F>(self, filter: F) -> Self::Filter<F>
    where
        F: Fn(&Self::Item) -> bool,
    {
        Vector(self.0.into_iter().filter(filter))
    }

    type FlatMap<Fm, Vo>
        = Vector<core::iter::FlatMap<I::IntoIter, Vo, Fm>>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo;
    #[inline(always)]
    fn flat_map<Fm, Vo>(self, flat_map: Fm) -> Self::FlatMap<Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo,
    {
        Vector(self.0.into_iter().flat_map(flat_map))
    }

    type FilterMap<Fm, O>
        = Vector<core::iter::FilterMap<I::IntoIter, Fm>>
    where
        Fm: Fn(Self::Item) -> Option<O>;
    #[inline(always)]
    fn filter_map<Fm, O>(self, filter_map: Fm) -> Self::FilterMap<Fm, O>
    where
        Fm: Fn(Self::Item) -> Option<O>,
    {
        Vector(self.0.into_iter().filter_map(filter_map))
    }

    type Whilst<W>
        = WhilstVector<VectorWhilstIter<I::IntoIter, W>, Self::Item>
    where
        W: Fn(&Self::Item) -> bool;
    fn whilst<W>(self, whilst: W) -> Self::Whilst<W>
    where
        W: Fn(&Self::Item) -> bool,
    {
        let iter = VectorWhilstIter::new(self.0.into_iter(), whilst);
        WhilstVector(iter)
    }

    type MapWhileOk<Mr, O, E>
        = VectorResult<core::iter::Map<I::IntoIter, Mr>, O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send;
    fn map_while_ok<Mr, O, E>(self, map_res: Mr) -> Self::MapWhileOk<Mr, O, E>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send,
    {
        let iter_res = self.0.into_iter().map(map_res);
        VectorResult(iter_res)
    }

    type UMap<U, M, O>
        = Vector<VectorUMapIter<I::IntoIter, U, M, O>>
    where
        M: Fn(*mut U, Self::Item) -> O;
    fn u_map<U, M, O>(self, u: *mut U, map: M) -> Self::UMap<U, M, O>
    where
        M: Fn(*mut U, Self::Item) -> O,
    {
        let iter = VectorUMapIter::new(u, self.0.into_iter(), map);
        Vector(iter)
    }

    type UFilter<U, F>
        = Vector<VectorUFilterIter<I::IntoIter, U, F>>
    where
        F: Fn(*mut U, &Self::Item) -> bool;
    fn u_filter<U, F>(self, u: *mut U, filter: F) -> Self::UFilter<U, F>
    where
        F: Fn(*mut U, &Self::Item) -> bool,
    {
        let iter = VectorUFilterIter::new(u, self.0.into_iter(), filter);
        Vector(iter)
    }

    type UFlatMap<U, Fm, Vo>
        = Vector<VectorUFlatMapIter<I::IntoIter, U, Fm, Vo>>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo;
    fn u_flat_map<U, Fm, Vo>(self, u: *mut U, flat_map: Fm) -> Self::UFlatMap<U, Fm, Vo>
    where
        Vo: IntoIterator,
        Fm: Fn(*mut U, Self::Item) -> Vo,
    {
        let iter = VectorUFlatMapIter::new(u, self.0.into_iter(), flat_map);
        Vector(iter)
    }

    type UFilterMap<U, Fm, O>
        = Vector<VectorUFilterMapIter<I::IntoIter, U, Fm, O>>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>;
    fn u_filter_map<U, Fm, O>(self, u: *mut U, filter_map: Fm) -> Self::UFilterMap<U, Fm, O>
    where
        Fm: Fn(*mut U, Self::Item) -> Option<O>,
    {
        let iter = VectorUFilterMapIter::new(u, self.0.into_iter(), filter_map);
        Vector(iter)
    }
}
