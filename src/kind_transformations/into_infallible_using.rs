use crate::infallible::Par;
use crate::infallible::Xap;
use crate::infallible_using::ParUsing;
use crate::infallible_using::using_var::UsingFun;
use crate::kind_transformations::IntoXapOpt;
use crate::option::ParOpt;
use crate::runner::ParRunner;
use orx_concurrent_iter::ConcurrentIter;

impl<I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    R: ParRunner,
{
    // pub fn fallible_option(self) -> ParOpt<I, X::XapOpt, R> {
    //     let (iter, xap, exe, params) = self.destruct();
    //     let xap = xap.into_xap_res();
    //     ParOpt::new(iter, xap, exe, params)
    // }

    pub fn using<U, F>(self, f: F)
    where
        F: Fn(usize) -> U + Sync,
    {
        let (iter, xap, exe, params) = self.destruct();
        let using = UsingFun::new(f);
        // ParUsing::new(using, iter, xap, exe, params);
        todo!()
    }
}
