use crate::infallible::XapCopied;
use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::par_iter::Par;
use crate::infallible::par_runner::ParRunnerInfallible;
use crate::infallible::xap::Xap;
use crate::infallible::xap_variants::Id;
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner, default_runner};
use orx_concurrent_iter::ConcurrentIter;

impl<'a, O: Copy + 'a, I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = &'a O>,
    R: ParRunner,
{
    pub fn copied(self) -> Par<I, X::Mapped<FnCopied<'a, O>>, R> {
        let (iter, xap, exe, params) = self.destruct();
        Par::new(iter, xap.mapped(FnCopied::new()), exe, params)
    }
}

impl<'a, O: Clone + 'a, I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = &'a O>,
    R: ParRunner,
{
    pub fn cloned(self) -> Par<I, X::Mapped<FnCloned<'a, O>>, R> {
        let (iter, xap, exe, params) = self.destruct();
        Par::new(iter, xap.mapped(FnCloned::new()), exe, params)
    }
}
