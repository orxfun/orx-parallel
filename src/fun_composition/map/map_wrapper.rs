use core::marker::PhantomData;

use crate::fun_composition::map::map_trait::Map;

pub struct FnMap<I, O, M>
where
    M: Fn(I) -> O,
{
    m: M,
    p: PhantomData<(I, O)>,
}

impl<I, O, M> FnMap<I, O, M>
where
    M: Fn(I) -> O,
{
    pub fn new(m: M) -> Self {
        let p = PhantomData;
        Self { m, p }
    }
}

impl<I, O, M> Map for FnMap<I, O, M>
where
    M: Fn(I) -> O,
{
    type I = I;

    type O = O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        (self.m)(i)
    }
}
