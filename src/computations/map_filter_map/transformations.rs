use super::mfm::Mfm;
use crate::computations::Values;
use orx_concurrent_iter::ConcurrentIter;

impl<I, T, Vt, O, Vo, M1, F, M2> Mfm<I, T, Vt, O, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values<Item = T>,
    O: Send + Sync,
    Vo: Values<Item = O>,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
{
    pub fn map<M, Q>(
        self,
        map: M,
    ) -> Mfm<I, T, Vt, Q, Vo::Mapped<M, Q>, M1, F, impl Fn(T) -> Vo::Mapped<M, Q>>
    where
        M: Fn(O) -> Q + Send + Sync + Clone,
        Q: Send + Sync,
    {
        let (params, iter, map1, filter, map2) = self.destruct();
        let map2 = move |t| {
            let vo = map2(t);
            vo.map(map.clone())
        };
        Mfm::new(params, iter, map1, filter, map2)
    }

    pub fn flat_map<Fm, Vq>(
        self,
        flat_map: Fm,
    ) -> Mfm<I, T, Vt, Vq::Item, Vo::FlatMapped<Fm, Vq>, M1, F, impl Fn(T) -> Vo::FlatMapped<Fm, Vq>>
    where
        Fm: Fn(O) -> Vq + Send + Sync + Clone,
        Vq: IntoIterator + Send + Sync,
        Vq::IntoIter: Send + Sync,
        Vq::Item: Send + Sync,
    {
        let (params, iter, map1, filter, map2) = self.destruct();
        let map2 = move |t| {
            let vo = map2(t);
            vo.flat_map(flat_map.clone())
        };
        Mfm::new(params, iter, map1, filter, map2)
    }
}
