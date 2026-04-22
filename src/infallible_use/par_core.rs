use crate::infallible_use::{Use, XapUse};
use crate::{Params, runner::ParRunner};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParUseCore {
    type Item;

    type Runner: ParRunner;

    type U;

    type Use: Use<Item = Self::U>;

    type Input: ConcurrentIter;

    type Xap: XapUse<
            U = <Self::Use as Use>::Item,
            I = <Self::Input as ConcurrentIter>::Item,
            O = Self::Item,
        >;

    fn destruct(self) -> (Self::Use, Self::Input, Self::Xap, Self::Runner, Params);
}
