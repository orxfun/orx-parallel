use crate::infallible_use::fun::flat_map::fn_trait::UFlatMap;
use core::marker::PhantomData;

pub struct UFnFlatMap<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send>(
    F,
    PhantomData<(I, U)>,
);

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> Clone for UFnFlatMap<U, I, O, F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> Copy for UFnFlatMap<U, I, O, F> {}

unsafe impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> Send
    for UFnFlatMap<U, I, O, F>
{
}

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> UFnFlatMap<U, I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> UFlatMap
    for UFnFlatMap<U, I, O, F>
{
    type I = I;

    type O = O;

    type U = U;

    #[inline(always)]
    fn flat_map(&self, u: &mut Self::U, i: Self::I) -> Self::O {
        (self.0)(u, i)
    }
}
