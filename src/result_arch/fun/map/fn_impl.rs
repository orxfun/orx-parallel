use crate::result_arch::fun::map::fn_trait::MapRes;
use core::marker::PhantomData;

// map

pub struct FnMapRes<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send>(F, PhantomData<I>);

impl<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send> Clone for FnMapRes<I, O, E, F> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send> Copy for FnMapRes<I, O, E, F> {}

unsafe impl<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send> Send for FnMapRes<I, O, E, F> {}

impl<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send> FnMapRes<I, O, E, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<I, O, E, F: Fn(I) -> Result<O, E> + Copy + Send> MapRes for FnMapRes<I, O, E, F> {
    type I = I;

    type O = O;

    type E = E;

    #[inline(always)]
    fn map_res(&self, i: Self::I) -> Result<Self::O, Self::E> {
        (self.0)(i)
    }
}
