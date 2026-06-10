use crate::Par;
use crate::infallible::{ParIter, XapEnumByInput};
use orx_concurrent_iter::{ConcurrentIter, enumerate::Enumerate};

pub trait EnumeratePar: Par {
    fn enumerate(self) -> impl Par<Item = (usize, Self::Item), Input = Enumerate<Self::Input>>;
}

impl<P> EnumeratePar for P
where
    P: Par,
    P::Xap: XapEnumByInput,
{
    fn enumerate(self) -> impl Par<Item = (usize, Self::Item), Input = Enumerate<Self::Input>>
    where
        Self::Xap: XapEnumByInput,
    {
        let (iter, xap, exe, params) = self.destruct();
        let iter = iter.enumerate();
        let xap = xap.enumerate();
        ParIter::new(iter, xap, exe, params)
    }
}
