use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::IntoConcurrentIter;

pub trait IntoParIter: IntoConcurrentIter {
    fn into_par(self) -> ParIter<Self::IntoIter, Id<Self::Item>>;
}

impl<I: IntoConcurrentIter> IntoParIter for I {
    fn into_par(self) -> ParIter<Self::IntoIter, Id<Self::Item>> {
        ParIter::new(
            self.into_con_iter(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}
