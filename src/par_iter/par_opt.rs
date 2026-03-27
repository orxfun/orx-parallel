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
