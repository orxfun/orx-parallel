use crate::computational_variants::xap_fn::{m::M, map::Map};
use core::marker::PhantomData;

pub struct M0<I> {
    p: PhantomData<I>,
}

impl<I> M0<I> {
    pub const fn new() -> Self {
        let p = PhantomData;
        Self { p }
    }
}

impl<I> Map<I, I> for M0<I> {
    #[inline(always)]
    fn map(&self, i: I) -> I {
        i
    }

    type Compose<Y, Q>
        = M<I, Q, Y>
    where
        Y: Fn(I) -> Q;

    fn compose<Y, Q>(self, y: Y) -> Self::Compose<Y, Q>
    where
        Y: Fn(I) -> Q,
    {
        M::new(y)
    }
}
