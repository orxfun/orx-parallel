use crate::result::fun::map::fn_trait::MapRes;
use core::marker::PhantomData;

// enter fallible

pub struct FnMapToRes<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send>(F, PhantomData<I>);

impl<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send> Clone for FnMapToRes<I, O, E, F> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send> Copy for FnMapToRes<I, O, E, F> {}

unsafe impl<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send> Send for FnMapToRes<I, O, E, F> {}

impl<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send> MapRes for FnMapToRes<I, O, E, F> {
    type I = I;

    type O = O;

    type E = E;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Result<Self::O, Self::E> {
        (self.0)(i)
    }
}
