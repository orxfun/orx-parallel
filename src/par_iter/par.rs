use crate::runner::{DefaultRunner, ParRunner};
use crate::xap::Xap;
use orx_concurrent_iter::ConcurrentIter;

pub struct Par<I, X, R = DefaultRunner>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    R: ParRunner,
{
    iter: I,
    xap: X,
    exe: R,
}
