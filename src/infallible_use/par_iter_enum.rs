#![allow(refining_impl_trait)]

use crate::ParUseIter;
use crate::infallible_use::{ParUse, XapUseEnumByInput};
use orx_concurrent_iter::{ConcurrentIter, enumerate::Enumerate};

pub trait ParUseIterEnumarable: ParUseIter {
    fn enumerate(self) -> impl ParUseIter<Use = Self::Use, Item = (usize, Self::Item)>;
}

impl<P> ParUseIterEnumarable for P
where
    P: ParUseIter,
    P::Xap: XapUseEnumByInput,
{
    fn enumerate(
        self,
    ) -> ParUse<
        Self::Use,
        Enumerate<Self::Input>,
        <Self::Xap as XapUseEnumByInput>::Enumerated,
        Self::Runner,
    >
    where
        Self::Xap: XapUseEnumByInput,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        let iter = iter.enumerate();
        let xap = xap.enumerate();
        ParUse::new(u, iter, xap, exe, params)
    }
}
