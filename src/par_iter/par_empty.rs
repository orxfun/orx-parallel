use super::par::Par;
use crate::parameters::Params;
use orx_concurrent_iter::ConcurrentIter;

pub struct ParEmpty<I>
where
    I: ConcurrentIter,
{
    iter: I,
    params: Params,
}

impl<I> Par for ParEmpty<I>
where
    I: ConcurrentIter,
{
    type Item = I::Item;

    // collect
}
