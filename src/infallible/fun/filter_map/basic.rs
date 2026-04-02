use crate::infallible::fun::FilterMap;
use core::marker::PhantomData;

pub struct FnFilMap<I, O, F: Fn(I) -> Option<O> + Copy + Send>(F, PhantomData<(I, O)>);

impl<I, O, F: Fn(I) -> Option<O> + Copy + Send> Clone for FnFilMap<I, O, F> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<I, O, F: Fn(I) -> Option<O> + Copy + Send> Copy for FnFilMap<I, O, F> {}

unsafe impl<I, O, F: Fn(I) -> Option<O> + Copy + Send> Send for FnFilMap<I, O, F> {}

impl<I, O, F: Fn(I) -> Option<O> + Copy + Send> FnFilMap<I, O, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O, F: Fn(I) -> Option<O> + Copy + Send> FilterMap for FnFilMap<I, O, F> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        (self.0)(i)
    }
}
