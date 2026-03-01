use crate::{
    computational_variants::xap_fn::{ff::FF, filter::Filter, map::Map, mm::MM, xap::XapFn},
    generic_values::Values,
};
use core::marker::PhantomData;

pub struct M<I, O, X>
where
    X: Fn(I) -> O,
{
    f: X,
    p: PhantomData<I>,
}

impl<I, O, X> M<I, O, X>
where
    X: Fn(I) -> O,
{
    pub fn new(f: X) -> Self {
        let p = PhantomData;
        Self { f, p }
    }
}

impl<I, O, X> Map<I, O> for M<I, O, X>
where
    X: Fn(I) -> O,
{
    #[inline(always)]
    fn map(&self, i: I) -> O {
        (self.f)(i)
    }

    type Compose<Y, Q>
        = MM<I, O, X, Q, M<O, Q, Y>>
    where
        Y: Fn(O) -> Q;

    fn compose<Y, Q>(self, y: Y) -> Self::Compose<Y, Q>
    where
        Y: Fn(O) -> Q,
    {
        MM::new(self.f, M::new(y))
    }
}
