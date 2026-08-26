use crate::common_par_traits::ParInfCommon;
use crate::infallible::recursive::par_core::ParRecCore;
use crate::infallible::{ParCore, ParIter, Xap};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::pool::ParThreadPool;
use crate::runner::{DefaultRunner, ParRunner};
use crate::{Par, ParCollectInto};
use orx_concurrent_iter::ConcurrentIter;

/// Parallel iterator.
pub struct ParIterRecursive<I, X, C, E, R = DefaultRunner>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    C: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> C + Send + Sync,
{
    iter: I,
    xap: X,
    exe: R,
    params: Params,
    extend: E,
}

impl<I, X, C, E, R> ParIterRecursive<I, X, C, E, R>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    C: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> C + Send + Sync,
{
    //
}

impl<I, X, C, E, R> ParRecCore for ParIterRecursive<I, X, C, E, R>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    C: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> C + Send + Sync,
{
    type Item = X::O;

    type Runner = R;

    type Input = I;

    type Xap = X;

    fn destruct(self) -> (Self::Input, Self::Xap, Self::Runner, Params) {
        (self.iter, self.xap, self.exe, self.params)
    }
}
