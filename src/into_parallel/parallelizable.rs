use crate::infallible::{Par, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::ConcurrentIterable;

pub trait Parallelizable: ConcurrentIterable {
    fn par(&self) -> Par<Self::Iter, Id<Self::Item>> {
        Par::new(
            self.con_iter(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}

impl<I> Parallelizable for I where I: ConcurrentIterable {}
