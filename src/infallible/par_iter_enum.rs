#![allow(refining_impl_trait)]

use crate::ParIter;
use crate::infallible::{Par, XapEnumByInput};
use orx_concurrent_iter::{ConcurrentIter, enumerate::Enumerate};

pub trait ParIterEnumarable: ParIter {
    fn enumerate(self) -> impl ParIter<Item = (usize, Self::Item)>;
}

impl<P> ParIterEnumarable for P
where
    P: ParIter,
    P::Xap: XapEnumByInput,
{
    fn enumerate(
        self,
    ) -> Par<Enumerate<Self::Input>, <Self::Xap as XapEnumByInput>::Enumerated, Self::Runner>
    where
        Self::Xap: XapEnumByInput,
    {
        let (iter, xap, exe, params) = self.destruct();
        let iter = iter.enumerate();
        let xap = xap.enumerate();
        Par::new(iter, xap, exe, params)
    }
}
