use crate::ParUse;
use crate::infallible_use::{ParUseIter, XapUseEnumByInput};
use orx_concurrent_iter::{ConcurrentIter, enumerate::Enumerate};

pub trait EnumerateParUse: ParUse {
    fn enumerate(
        self,
    ) -> impl ParUse<
        Using = Self::Using,
        Use = Self::Use,
        Item = (usize, Self::Item),
        Input = Enumerate<Self::Input>,
    >;
}

impl<P> EnumerateParUse for P
where
    P: ParUse,
    P::Xap: XapUseEnumByInput,
{
    fn enumerate(
        self,
    ) -> impl ParUse<
        Using = Self::Using,
        Use = Self::Use,
        Item = (usize, Self::Item),
        Input = Enumerate<Self::Input>,
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
