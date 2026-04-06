use crate::infallible::Par;
use crate::infallible::Xap;
use crate::kind_transformations::IntoXapOpt;
use crate::option::ParOpt;
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;

impl<O, I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = Option<O>> + IntoXapOpt,
    R: ParRunner,
{
    pub fn fallible_option(self) -> ParOpt<I, X::XapOpt, R> {
        let (iter, xap, exe, params) = self.destruct();
        let xap = xap.into_xap_res();
        ParOpt::new(iter, xap, exe, params)
    }
}
