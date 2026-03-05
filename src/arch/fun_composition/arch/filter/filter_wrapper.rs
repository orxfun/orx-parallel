use crate::fun_composition::filter::filter_trait::Filter;
use core::marker::PhantomData;

pub struct FnFilter<I, F: Fn(&I) -> bool> {
    f: F,
    p: PhantomData<I>,
}

impl<I, F: Fn(&I) -> bool> FnFilter<I, F> {
    pub fn new(f: F) -> Self {
        let p = PhantomData;
        Self { f, p }
    }
}

impl<I, F: Fn(&I) -> bool> Filter for FnFilter<I, F> {
    type I = I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        (self.f)(i)
    }
}
