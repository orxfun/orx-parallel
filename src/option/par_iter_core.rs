use crate::{Params, infallible::Xap, runner::ParRunner, sizes::SizePair};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParOptIterCore {
    type Item;

    type Runner: ParRunner;

    type Input: ConcurrentIter;

    type M;

    type Xap1: Xap<I = <Self::Input as ConcurrentIter>::Item, O = Option<Self::M>>;

    type Xap2: Xap<I = Self::M, O = Self::Item>;

    type SizePair: SizePair<S1 = <Self::Xap1 as Xap>::Size, S2 = <Self::Xap2 as Xap>::Size>;

    fn destruct(
        self,
    ) -> (
        Self::Input,
        Self::Xap1,
        Self::Xap2,
        Self::Runner,
        Self::SizePair,
        Params,
    );
}
