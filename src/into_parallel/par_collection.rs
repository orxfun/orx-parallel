use crate::infallible::{Par, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::{ConcurrentCollection, ConcurrentIterable};

pub trait ParCol: ConcurrentCollection {
    fn par(&self) -> Par<<Self::Iterable<'_> as ConcurrentIterable>::Iter, Id<&Self::Item>> {
        Par::new(
            self.con_iter(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}

impl<X> ParCol for X where X: ConcurrentCollection {}
