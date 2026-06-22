use crate::infallible::fun::FlatMap;
use core::marker::PhantomData;

pub struct FnFlatten<I: IntoIterator>(PhantomData<I>);

impl<I: IntoIterator> Clone for FnFlatten<I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I: IntoIterator> Copy for FnFlatten<I> {}

unsafe impl<I: IntoIterator> Send for FnFlatten<I> {}

impl<I: IntoIterator> FnFlatten<I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<I: IntoIterator> FlatMap for FnFlatten<I> {
    type I = I;

    type O = I::IntoIter;

    #[inline(always)]
    fn flat_map(&self, i: Self::I) -> Self::O {
        i.into_iter()
    }
}
