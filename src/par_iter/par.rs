use crate::runner::{DefaultRunner, ParRunner, default_runner};
use crate::xap::{Id, Xap};
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

// TODO: this will be replaced later by IntoPar trait.
pub fn par<I: ConcurrentIter>(iter: I) -> Par<I, Id<I::Item>, DefaultRunner> {
    Par {
        iter,
        xap: Id::new(),
        exe: default_runner(),
    }
}
