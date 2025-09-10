use crate::orch::Orchestrator;

use super::m::M;
use orx_concurrent_iter::ConcurrentIter;

impl<R, I, O, M1> M<R, I, O, M1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O,
{
    pub fn map<M2, Q>(self, map: M2) -> M<R, I, Q, impl Fn(I::Item) -> Q>
    where
        M2: Fn(O) -> Q,
        Q: Send,
    {
        let (orchestrator, params, iter, map1) = self.destruct();
        let map2 = move |t| map(map1(t));
        M::new(orchestrator, params, iter, map2)
    }
}
