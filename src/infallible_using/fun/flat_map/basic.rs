use crate::infallible_using::fun::flat_map::fn_trait::FlatMapU;
use core::marker::PhantomData;

pub struct FnFlatMapU<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send>(
    F,
    PhantomData<(I, U)>,
);

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> Clone for FnFlatMapU<U, I, O, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> Copy for FnFlatMapU<U, I, O, F> {}

unsafe impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> Send
    for FnFlatMapU<U, I, O, F>
{
}

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> FnFlatMapU<U, I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<U, I, O: IntoIterator, F: Fn(&mut U, I) -> O + Copy + Send> FlatMapU
    for FnFlatMapU<U, I, O, F>
{
    type I = I;

    type O = O;

    type U = U;

    #[inline(always)]
    fn flat_map(&self, u: &mut Self::U, i: Self::I) -> Self::O {
        (self.0)(u, i)
    }
}
