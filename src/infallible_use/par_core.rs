use crate::infallible_use::{Using, XapUse};
use crate::{Params, runner::ParRunner};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParUseCore {
    type Item;

    type Runner: ParRunner;

    type Use;

    type Using: Using<Item = Self::Use>;

    type Input: ConcurrentIter;

    type Xap: XapUse<U = Self::Use, I = <Self::Input as ConcurrentIter>::Item, O = Self::Item>;

    fn destruct(self) -> (Self::Using, Self::Input, Self::Xap, Self::Runner, Params);
}
