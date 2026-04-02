use crate::infallible_using::fun::filter_map::fn_trait::FilterMapU;
use core::marker::PhantomData;

pub struct FnFilMapU<U, I, O, F: Fn(&mut U, I) -> Option<O> + Copy + Send>(F, PhantomData<(I, U)>);

impl<U, I, O, F: Fn(&mut U, I) -> Option<O> + Copy + Send> Clone for FnFilMapU<U, I, O, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<U, I, O, F: Fn(&mut U, I) -> Option<O> + Copy + Send> Copy for FnFilMapU<U, I, O, F> {}

unsafe impl<U, I, O, F: Fn(&mut U, I) -> Option<O> + Copy + Send> Send for FnFilMapU<U, I, O, F> {}

impl<U, I, O, F: Fn(&mut U, I) -> Option<O> + Copy + Send> FnFilMapU<U, I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<U, I, O, F: Fn(&mut U, I) -> Option<O> + Copy + Send> FilterMapU for FnFilMapU<U, I, O, F> {
    type I = I;

    type O = O;

    type U = U;

    #[inline(always)]
    fn filter_map(&self, u: &mut Self::U, i: Self::I) -> Option<Self::O> {
        (self.0)(u, i)
    }
}
