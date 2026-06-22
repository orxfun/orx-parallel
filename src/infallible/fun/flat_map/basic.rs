use crate::infallible::fun::FlatMap;
use core::marker::PhantomData;

pub struct FnFlatMap<I, O: IntoIterator, F: Fn(I) -> O + Copy + Send>(F, PhantomData<(I, O)>);

impl<I, O: IntoIterator, F: Fn(I) -> O + Copy + Send> Clone for FnFlatMap<I, O, F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I, O: IntoIterator, F: Fn(I) -> O + Copy + Send> Copy for FnFlatMap<I, O, F> {}

unsafe impl<I, O: IntoIterator, F: Fn(I) -> O + Copy + Send> Send for FnFlatMap<I, O, F> {}

impl<I, O: IntoIterator, F: Fn(I) -> O + Copy + Send> FnFlatMap<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O: IntoIterator, F: Fn(I) -> O + Copy + Send> FlatMap for FnFlatMap<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn flat_map(&self, i: Self::I) -> Self::O {
        (self.0)(i)
    }
}
