use crate::collectables::Collectable;
use crate::collectables::alg::merge_collected::merge_arb;
use crate::collectables::inf::ColIntoInf;
use crate::infallible_use::XapUse;
use crate::result_use::{ParRunnerUseRes, ParUseResultCore, ParUseResultIter};
use crate::sizes::SizePair;
use crate::use_var::Use;
use orx_concurrent_iter::ConcurrentIter;

pub trait ColIntoResUse<T>: Sized {
    fn res_use_col_into<U, I, M, E, X1, X2, S, R>(
        dst: &mut Self,
        par: ParUseResultIter<U, I, M, E, X1, X2, S, R>,
    ) -> Result<(), E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseRes,
        T: Send,
        E: Send;

    fn res_use_arb_col_into<U, I, M, E, X1, X2, S, R>(
        dst: &mut Self,
        par: ParUseResultIter<U, I, M, E, X1, X2, S, R>,
    ) -> Result<(), E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M, O = T>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        R: ParRunnerUseRes,
        T: Send,
        E: Send,
        Self: ColIntoInf<T> + Collectable<T>,
    {
        let (u, iter, x1, x2, mut exe, s, params) = par.destruct();
        let results = exe
            .collect_arb::<_, _, _, _, _, _, _, <Self as ColIntoInf<T>>::ThreadColArb>(
                s, params, u, iter, x1, x2,
            );
        results.map(|results| merge_arb(results, dst))
    }
}
