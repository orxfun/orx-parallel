use crate::{
    computational_variants::xap_fn::{ff::FF, filter::Filter, map::Map, xap::XapFn},
    generic_values::Values,
};
use core::marker::PhantomData;

pub struct MM<I, O1, X1, O2, X2>
where
    X1: Fn(I) -> O1,
    X2: Map<O1, O2>,
{
    f1: X1,
    f2: X2,
    p: PhantomData<(I, O2)>,
}

impl<I, O1, X1, O2, X2> MM<I, O1, X1, O2, X2>
where
    X1: Fn(I) -> O1,
    X2: Map<O1, O2>,
{
    pub fn new(f1: X1, f2: X2) -> Self {
        let p = PhantomData;
        Self { f1, f2, p }
    }
}

impl<I, O1, X1, O2, X2> Map<I, O2> for MM<I, O1, X1, O2, X2>
where
    X1: Fn(I) -> O1,
    X2: Map<O1, O2>,
{
    #[inline(always)]
    fn map(&self, i: I) -> O2 {
        let x = (self.f1)(i);
        self.f2.map(x)
    }

    type Compose<Y, Q>
        = MM<I, O1, X1, Q, X2::Compose<Y, Q>>
    where
        Y: Fn(O2) -> Q;

    fn compose<Y, Q>(self, y: Y) -> Self::Compose<Y, Q>
    where
        Y: Fn(O2) -> Q,
    {
        MM::new(self.f1, self.f2.compose(y))
    }
}
