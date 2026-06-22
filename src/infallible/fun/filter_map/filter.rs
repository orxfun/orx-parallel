use crate::infallible::fun::FilterMap;
use core::marker::PhantomData;

pub struct FnFil<I, F: Fn(&I) -> bool + Copy + Send>(F, PhantomData<I>);

impl<I, F: Fn(&I) -> bool + Copy + Send> Clone for FnFil<I, F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I, F: Fn(&I) -> bool + Copy + Send> Copy for FnFil<I, F> {}

unsafe impl<I, F: Fn(&I) -> bool + Copy + Send> Send for FnFil<I, F> {}

impl<I, F: Fn(&I) -> bool + Copy + Send> FnFil<I, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, F: Fn(&I) -> bool + Copy + Send> FilterMap for FnFil<I, F> {
    type I = I;

    type O = I;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        match (self.0)(&i) {
            true => Some(i),
            false => None,
        }
    }
}
