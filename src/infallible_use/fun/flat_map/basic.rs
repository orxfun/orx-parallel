use crate::infallible_use::fun::flat_map::fn_trait::FlatMap;
use core::marker::PhantomData;

pub struct FnFlatMap<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send>(
    F,
    PhantomData<(I, U)>,
);

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> Clone for FnFlatMap<U, I, O, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> Copy for FnFlatMap<U, I, O, F> {}

unsafe impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> Send
    for FnFlatMap<U, I, O, F>
{
}

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> FnFlatMap<U, I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> FlatMap for FnFlatMap<U, I, O, F> {
    type I = I;

    type O = O;

    type U = U;

    #[inline(always)]
    fn flat_map(&self, u: &mut Self::U, i: Self::I) -> Self::O {
        (self.0)(u, i)
    }
}
