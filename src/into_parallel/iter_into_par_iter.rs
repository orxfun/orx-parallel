use crate::infallible::{Par, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::{IterIntoConcurrentIter, implementations::ConIterOfIter};

pub trait IterIntoParIter: Iterator {
    fn iter_into_par(self) -> Par<ConIterOfIter<Self>, Id<Self::Item>>
    where
        Self: Sized,
        Self::Item: Send;
}

impl<I> IterIntoParIter for I
where
    I: Iterator,
    I::Item: Send + Sync,
{
    fn iter_into_par(self) -> Par<ConIterOfIter<Self>, Id<Self::Item>> {
        Par::new(
            self.iter_into_con_iter(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}
