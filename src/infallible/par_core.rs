use crate::{Params, infallible::Xap, runner::ParRunner};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParCore {
    type Item;

    type Runner: ParRunner;

    type Input: ConcurrentIter;

    type Xap: Xap<I = <Self::Input as ConcurrentIter>::Item, O = Self::Item>;

    fn destruct(self) -> (Self::Input, Self::Xap, Self::Runner, Params);
}
