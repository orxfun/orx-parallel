use crate::{computational_variants::Par, runner::DefaultRunner, Params};
use orx_concurrent_iter::{ConcurrentCollection, ConcurrentIterable};

pub trait ParallelizableCollection: ConcurrentCollection {
    type Item;

    fn par(
        &self,
    ) -> Par<
        <<Self as ConcurrentCollection>::Iterable<'_> as ConcurrentIterable>::Iter,
        DefaultRunner,
    > {
        Par::new(Params::default(), self.con_iter())
    }
}

impl<X> ParallelizableCollection for X
where
    X: ConcurrentCollection,
{
    type Item = <Self as ConcurrentCollection>::Item;
}
