use crate::infallible::{Par, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::IntoConcurrentIter;

pub trait IntoParIter: IntoConcurrentIter {
    fn into_par(self) -> Par<Self::IntoIter, Id<Self::Item>>;
}

impl<I: IntoConcurrentIter> IntoParIter for I {
    fn into_par(self) -> Par<Self::IntoIter, Id<Self::Item>> {
        Par::new(
            self.into_con_iter(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}
