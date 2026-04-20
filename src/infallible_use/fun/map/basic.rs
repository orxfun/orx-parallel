use crate::infallible_use::fun::map::fn_trait::UMap;
use core::marker::PhantomData;

pub struct UFnMap<U, I, O, F: Fn(&mut U, I) -> O + Copy + Send>(F, PhantomData<(I, U)>);

impl<U, I, O, F: Fn(&mut U, I) -> O + Copy + Send> Clone for UFnMap<U, I, O, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<U, I, O, F: Fn(&mut U, I) -> O + Copy + Send> Copy for UFnMap<U, I, O, F> {}

unsafe impl<U, I, O, F: Fn(&mut U, I) -> O + Copy + Send> Send for UFnMap<U, I, O, F> {}

impl<U, I, O, F: Fn(&mut U, I) -> O + Copy + Send> UFnMap<U, I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<U, I, O, F: Fn(&mut U, I) -> O + Copy + Send> UMap for UFnMap<U, I, O, F> {
    type I = I;

    type O = O;

    type U = U;

    #[inline(always)]
    fn map(&self, u: &mut Self::U, i: Self::I) -> Self::O {
        (self.0)(u, i)
    }
}
