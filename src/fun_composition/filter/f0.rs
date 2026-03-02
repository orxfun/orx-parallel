use crate::fun_composition::filter::filter_trait::Filter;
use core::marker::PhantomData;

pub struct F0<T> {
    p: PhantomData<T>,
}

impl<T> F0<T> {
    pub const fn new() -> Self {
        let p = PhantomData;
        Self { p }
    }
}

impl<T> Filter for F0<T> {
    type I = T;

    #[inline(always)]
    fn filter(&self, _: &Self::I) -> bool {
        true
    }
}
