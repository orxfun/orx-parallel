use crate::infallible_using::fun::filter_map::fn_trait::FilterMapU;
use core::marker::PhantomData;

pub struct FnFilU<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send>(F, PhantomData<(I, U)>);

impl<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send> Clone for FnFilU<U, I, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send> Copy for FnFilU<U, I, F> {}

unsafe impl<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send> Send for FnFilU<U, I, F> {}

impl<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send> FnFilU<U, I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<U, I, F: Fn(&mut U, &I) -> bool + Copy + Send> FilterMapU for FnFilU<U, I, F> {
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
