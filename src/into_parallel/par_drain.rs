use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use core::ops::RangeBounds;
use orx_concurrent_iter::ConcurrentDrainableOverSlice;

pub trait ParDrain: ConcurrentDrainableOverSlice {
    fn par_drain<R>(
        &mut self,
        range: R,
    ) -> ParIter<<Self as ConcurrentDrainableOverSlice>::DrainingIter<'_>, Id<Self::Item>>
    where
        R: RangeBounds<usize>,
    {
        ParIter::new(
            self.con_drain(range),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}

impl<I> ParDrain for I where I: ConcurrentDrainableOverSlice {}
