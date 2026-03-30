use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::par_iter::Par;
use crate::infallible::xap::Xap;
use crate::into_fallible::IntoXapRes;
use crate::result::ParRes;
use crate::runner::ParRunner;
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

impl<O, E, I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Result<O, E>> + IntoXapRes,
    R: ParRunner,
{
    pub fn into_result(self) -> ParRes<I, X::XapRes, R> {
        let (iter, xap, exe, params) = self.destruct();
        let xap = xap.into_xap_res();
        ParRes::new(iter, xap, exe, params)
    }
}
