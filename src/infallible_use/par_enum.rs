#![allow(refining_impl_trait)]

use crate::ParUse;
use crate::infallible_use::{ParUseIter, XapUseEnumByInput};
use orx_concurrent_iter::{ConcurrentIter, enumerate::Enumerate};

pub trait EnumerateParUse: ParUse {
    fn enumerate(self) -> impl ParUse<Use = Self::Use, U = Self::U, Item = (usize, Self::Item)>;
}

impl<P> EnumerateParUse for P
where
    P: ParUse,
    P::Xap: XapUseEnumByInput,
{
    fn enumerate(
        self,
    ) -> ParUseIter<
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
        ParUseIter::new(u, iter, xap, exe, params)
    }
}
