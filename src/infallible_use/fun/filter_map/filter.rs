use crate::infallible_use::fun::filter_map::fn_trait::FilterMap;
use core::marker::PhantomData;

pub struct FnFil<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send>(F, PhantomData<(I, U)>);

impl<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send> Clone for FnFil<U, I, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send> Copy for FnFil<U, I, F> {}

unsafe impl<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send> Send for FnFil<U, I, F> {}

impl<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send> FnFil<U, I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send> FilterMap for FnFil<U, I, F> {
    type I = I;

    type O = I;

    type U = U;

    #[inline(always)]
    fn filter_map(&self, u: &mut Self::U, i: Self::I) -> Option<Self::O> {
        match (self.0)(u, &i) {
            true => Some(i),
            false => None,
        }
    }
}
