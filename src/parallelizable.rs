use crate::{par_iter::ParEmpty, parameters::Params};
use orx_concurrent_iter::{ConcurrentIter, ConcurrentIterable};

pub trait Parallelizable {
    type ConcurrentIter: ConcurrentIter;

    fn par(&self) -> ParEmpty<Self::ConcurrentIter>;
}

impl<I> Parallelizable for I
where
    I: ConcurrentIterable,
{
    type ConcurrentIter = I::Iter;

    fn par(&self) -> ParEmpty<Self::ConcurrentIter> {
        ParEmpty::new(self.concurrent_iter(), Params::default())
    }
}
