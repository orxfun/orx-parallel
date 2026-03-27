use crate::Par;
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner, default_runner};
use crate::xap::{Id, Xap};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParOpt<I, X, O, R = DefaultRunner>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Option<O>>,
    R: ParRunner,
{
    par: Par<I, X, R>,
}

impl<I, X, O, R> ParOpt<I, X, O, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Option<O>>,
    R: ParRunner,
{
    // transformations

    pub fn map<Q, H>(self, h: H) -> Par<I, X::Map<Q, H>, R>
    where
        H: Fn(X::O) -> Q + Copy + Send,
    {
        todo!()
        // let xap = self.xap.map(h);
        // self.with_xap(xap)
    }
}
