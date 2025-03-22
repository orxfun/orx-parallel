use super::{
    mfm::Mfm,
    values::{Atom, Values},
};
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
    pub fn map<Map, Q>(
        self,
        map: Map,
    ) -> Mfm<I, T, Vt, Q, Vo::Mapped<Map, Q>, M1, F, impl Fn(T) -> Vo::Mapped<Map, Q>>
    where
        Map: Fn(O) -> Q + Send + Sync + Clone,
        Q: Send + Sync,
    {
        let (params, iter, map1, filter, map2) = self.destruct();
        let map2 = move |t| {
            let vo = map2(t);
            vo.map(map.clone())
        };
        Mfm::new(params, iter, map1, filter, map2)
    }
}
