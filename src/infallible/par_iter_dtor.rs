use crate::{Params, infallible::Xap, runner::ParRunner};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParIterDestruct {
    type Runner: ParRunner;

    type Input: ConcurrentIter;

    type Xap: Xap<I = <Self::Input as ConcurrentIter>::Item>;

    fn destruct(self) -> (Self::Input, Self::Xap, Self::Runner, Params);
}
