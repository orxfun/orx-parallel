use crate::{
    Params,
    infallible_use::{Use, XapUse},
    runner::ParRunner,
    sizes::SizePair,
};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParUseResIterCore {
    type Item;

    type Error;

    type Runner: ParRunner;

    type Use: Use;

    type Input: ConcurrentIter;

    type M;

    type Xap1: XapUse<
            U = <Self::Use as Use>::Item,
            I = <Self::Input as ConcurrentIter>::Item,
            O = Result<Self::M, Self::Error>,
        >;

    type Xap2: XapUse<U = <Self::Use as Use>::Item, I = Self::M, O = Self::Item>;

    type Size: SizePair<S1 = <Self::Xap1 as XapUse>::Size, S2 = <Self::Xap2 as XapUse>::Size>;

    fn destruct(
        self,
    ) -> (
        Self::Use,
        Self::Input,
        Self::Xap1,
        Self::Xap2,
        Self::Runner,
        Self::Size,
        Params,
    );
}
