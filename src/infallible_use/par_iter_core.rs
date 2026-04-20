use crate::{Params, infallible::Xap, infallible_use::Use, runner::ParRunner};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParUseIterCore {
    type Item;

    type Runner: ParRunner;

    type Use: Use;

    type Input: ConcurrentIter;

    type Xap: Xap<I = <Self::Input as ConcurrentIter>::Item, O = Self::Item>;

    fn destruct(self) -> (Self::Use, Self::Input, Self::Xap, Self::Runner, Params);
}
