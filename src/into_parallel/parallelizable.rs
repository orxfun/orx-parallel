use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::ConcurrentIterable;

pub trait Parallelizable: ConcurrentIterable {
    fn par(&self) -> ParIter<Self::Iter, Id<Self::Item>> {
        ParIter::new(
            self.con_iter(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}

impl<I> Parallelizable for I where I: ConcurrentIterable {}
