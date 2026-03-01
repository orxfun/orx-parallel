use crate::{
    computational_variants::xap_fn::{f::F, filter::Filter, xap::XapFn},
    generic_values::Values,
};
use core::marker::PhantomData;

pub struct FF<I, X1, X2>
where
    X1: Fn(&I) -> bool,
    X2: Filter<I>,
{
    f: X1,
    b: X2,
    p: PhantomData<I>,
}

impl<I, X1, X2> FF<I, X1, X2>
where
    X1: Fn(&I) -> bool,
    X2: Filter<I>,
{
    pub fn new(f: X1, b: X2) -> Self {
        let p = PhantomData;
        Self { f, b, p }
    }
}

impl<I, X1, X2> Filter<I> for FF<I, X1, X2>
where
    X1: Fn(&I) -> bool,
    X2: Filter<I>,
{
    #[inline(always)]
    fn filter(&self, i: &I) -> bool {
        (self.f)(i) && self.b.filter(i)
    }

    type Compose<Y>
        = FF<I, X1, X2::Compose<Y>>
    where
        Y: Fn(&I) -> bool;

    fn compose<Y: Fn(&I) -> bool>(self, y: Y) -> Self::Compose<Y> {
        FF::new(self.f, self.b.compose(y))
    }
}
