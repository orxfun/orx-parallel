use crate::infallible::{Par, xap_variants::Id};
use crate::into_parallel::par_collection::ParCol;
use crate::runner::default_runner;
use orx_concurrent_iter::ConcurrentCollectionMut;

pub trait ParColMut: ConcurrentCollectionMut + ParCol {
    fn par_mut(&mut self) -> Par<Self::IterMut<'_>, Id<&mut Self::Item>> {
        Par::new(
            self.con_iter_mut(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}

impl<X> ParColMut for X where X: ConcurrentCollectionMut + ParCol {}
