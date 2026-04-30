use crate::infallible_use::fun::UFlatMap;
use core::marker::PhantomData;

pub struct UFnFlatten<U, I: IntoIterator>(PhantomData<(U, I)>);

impl<U, I: IntoIterator> Clone for UFnFlatten<U, I> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<U, I: IntoIterator> Copy for UFnFlatten<U, I> {}

unsafe impl<U, I: IntoIterator> Send for UFnFlatten<U, I> {}

impl<U, I: IntoIterator> UFnFlatten<U, I> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<U, I: IntoIterator> UFlatMap for UFnFlatten<U, I> {
    type U = U;

    type I = I;

    type O = I::IntoIter;

    #[inline(always)]
    fn flat_map(&self, _: &mut Self::U, i: Self::I) -> Self::O {
        i.into_iter()
    }
}
