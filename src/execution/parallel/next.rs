use crate::execution::{parallel::par_comp::ParComp, val_and_idx::ValIdx};
use crate::parameters::Params;
use crate::xap::Xap;
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn next<Q, I, X>(exe: &mut Q, iter: &I, x: X, params: Params) -> Option<ValIdx<X::O>>
where
    Q: ParComp,
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
{
    todo!()
}
