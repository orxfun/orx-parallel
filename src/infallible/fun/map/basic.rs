use crate::infallible::fun::Map;
use core::marker::PhantomData;

pub struct FnMap<I, O, F: Fn(I) -> O + Copy + Send>(F, PhantomData<I>);

impl<I, O, F: Fn(I) -> O + Copy + Send> Clone for FnMap<I, O, F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I, O, F: Fn(I) -> O + Copy + Send> Copy for FnMap<I, O, F> {}

unsafe impl<I, O, F: Fn(I) -> O + Copy + Send> Send for FnMap<I, O, F> {}

impl<I, O, F: Fn(I) -> O + Copy + Send> FnMap<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O, F: Fn(I) -> O + Copy + Send> Map for FnMap<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.0)(i)
    }
}
