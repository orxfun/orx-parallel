use super::m::M;
use orx_concurrent_iter::ConcurrentIter;

impl<I, O, M1> M<I, O, M1>
where
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O,
{
    pub fn map<M2, Q>(self, map: M2) -> M<I, Q, impl Fn(I::Item) -> Q>
    where
        M2: Fn(O) -> Q,
        Q: Send,
    {
        let (params, iter, map1) = self.destruct();
        let map2 = move |t| map(map1(t));
        M::new(params, iter, map2)
    }
}
