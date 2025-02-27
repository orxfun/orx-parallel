use super::par::{Par, ParCore};
use crate::{collect_into::ParCollectInto, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParEmpty<I>
where
    I: ConcurrentIter,
{
    iter: I,
    params: Params,
}

impl<I: ConcurrentIter> ParCore for ParEmpty<I> {
    fn input_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }
}

impl<I> Par for ParEmpty<I>
where
    I: ConcurrentIter,
{
    type Item = I::Item;

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        todo!()
    }
}
