use crate::fun_composition::map::map_trait::Map;
use core::marker::PhantomData;

pub struct FnMap<I, O, M: Fn(I) -> O> {
    m: M,
    p: PhantomData<(I, O)>,
}

impl<I, O, M: Fn(I) -> O> FnMap<I, O, M> {
    pub fn new(m: M) -> Self {
        let p = PhantomData;
        Self { m, p }
    }
}

impl<I, O, M: Fn(I) -> O> Map for FnMap<I, O, M> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.m)(i)
    }
}
