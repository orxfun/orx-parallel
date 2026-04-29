use crate::infallible::fun::Map;
use core::marker::PhantomData;

pub struct FnFlatten<I: IntoIterator>(PhantomData<I>);

impl<I: IntoIterator> Clone for FnFlatten<I> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<I: IntoIterator> Copy for FnFlatten<I> {}

unsafe impl<I: IntoIterator> Send for FnFlatten<I> {}

impl<I: IntoIterator> Map for FnFlatten<I> {
    type I = I;

    type O = I::IntoIter;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        i.into_iter()
    }
}
