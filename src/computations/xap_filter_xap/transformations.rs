use super::xfx::Xfx;
use crate::computations::Values;
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vt, Vo, M1, F, M2> Xfx<I, Vt, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values,
    Vo: Values,
    M1: Fn(I::Item) -> Vt + Sync,
    F: Fn(&Vt::Item) -> bool + Sync,
    M2: Fn(Vt::Item) -> Vo + Sync,
{
    #[allow(clippy::type_complexity)]
    pub fn map<M, Q>(
        self,
        map: M,
    ) -> Xfx<I, Vt, Vo::Mapped<M, Q>, M1, F, impl Fn(Vt::Item) -> Vo::Mapped<M, Q>>
    where
        M: Fn(Vo::Item) -> Q + Send + Sync + Clone,
        Q: Send + Sync,
    {
        let (params, iter, map1, filter, map2) = self.destruct();
        let map2 = move |t| {
            let vo = map2(t);
            vo.map(map.clone())
        };
        Xfx::new(params, iter, map1, filter, map2)
    }

    #[allow(clippy::type_complexity)]
    pub fn flat_map<Fm, Vq>(
        self,
        flat_map: Fm,
    ) -> Xfx<I, Vt, Vo::FlatMapped<Fm, Vq>, M1, F, impl Fn(Vt::Item) -> Vo::FlatMapped<Fm, Vq>>
    where
        Fm: Fn(Vo::Item) -> Vq + Send + Sync + Clone,
        Vq: IntoIterator + Send + Sync,
        Vq::IntoIter: Send + Sync,
        Vq::Item: Send + Sync,
    {
        let (params, iter, map1, filter, map2) = self.destruct();
        let map2 = move |t| {
            let vo = map2(t);
            vo.flat_map(flat_map.clone())
        };
        Xfx::new(params, iter, map1, filter, map2)
    }
}
