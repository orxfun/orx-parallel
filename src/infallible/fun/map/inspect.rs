use crate::infallible::fun::Map;
use core::marker::PhantomData;

/// Inspect adapter that returns the input unchanged.
pub struct FnIns<I, F: Fn(&I) + Copy + Send>(F, PhantomData<I>);

impl<I, F: Fn(&I) + Copy + Send> Clone for FnIns<I, F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I, F: Fn(&I) + Copy + Send> Copy for FnIns<I, F> {}

unsafe impl<I, F: Fn(&I) + Copy + Send> Send for FnIns<I, F> {}

impl<I, F: Fn(&I) + Copy + Send> FnIns<I, F> {
    /// Creates an inspect adapter.
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, F: Fn(&I) + Copy + Send> Map for FnIns<I, F> {
    type I = I;

    type O = I;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.0)(&i);
        i
    }
}
