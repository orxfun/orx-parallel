use crate::using::Using;
use crate::using::computations::UM;
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, O, M1> UM<U, I, O, M1>
where
    U: Using,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync,
{
    #[allow(clippy::type_complexity)]
    pub fn map<M2, Q>(self, map: M2) -> UM<U, I, Q, impl Fn(&mut U::Item, I::Item) -> Q>
    where
        M2: Fn(&mut U::Item, O) -> Q + Send + Sync,
        Q: Send + Sync,
    {
        let (using, params, iter, map1) = self.destruct();
        let map2 = move |u: &mut U::Item, t: <I as ConcurrentIter>::Item| {
            let v1 = map1(u, t);
            map(u, v1)
        };
        UM::new(using, params, iter, map2)
    }
}
