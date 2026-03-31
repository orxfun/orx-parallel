use crate::fun_composition::map::map_trait::Map;
use core::marker::PhantomData;

pub struct M0<T> {
    p: PhantomData<T>,
}

impl<T> M0<T> {
    pub const fn new() -> Self {
        let p = PhantomData;
        Self { p }
    }
}

impl<T> Map for M0<T> {
    type I = T;

    type O = T;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        i
    }
}
