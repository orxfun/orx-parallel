use crate::{
    DefaultRunner, ParIter, ParallelizableCollection, Params, computational_variants::Par,
};
use orx_concurrent_iter::ConcurrentCollectionMut;

pub trait ParallelizableCollectionMut: ConcurrentCollectionMut + ParallelizableCollection {
    fn par_mut(&mut self) -> impl ParIter<DefaultRunner, Item = &mut Self::Item> {
        Par::new(Params::default(), self.con_iter_mut())
    }
}

impl<X> ParallelizableCollectionMut for X where X: ConcurrentCollectionMut + ParallelizableCollection
{}
