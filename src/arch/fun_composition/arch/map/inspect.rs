use crate::fun_composition::map::map_trait::Map;
use core::marker::PhantomData;

pub struct Ins<T, I: Fn(&T)> {
    i: I,
    p: PhantomData<T>,
}

impl<T, I: Fn(&T)> Ins<T, I> {
    pub fn new(i: I) -> Self {
        let p = PhantomData;
        Self { i, p }
    }
}

impl<T, I: Fn(&T)> Map for Ins<T, I> {
    type I = T;

    type O = T;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.i)(&i);
        i
    }
}
