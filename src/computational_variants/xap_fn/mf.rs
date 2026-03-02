use crate::computational_variants::xap_fn::{map_filter::MapFilter, mfmf::MFMF};
use core::marker::PhantomData;

pub struct MF<I, O, X, Y>
where
    X: Fn(I) -> O,
    Y: Fn(&O) -> bool,
{
    m: X,
    f: Y,
    p: PhantomData<I>,
}

impl<I, O, X, Y> MF<I, O, X, Y>
where
    X: Fn(I) -> O,
    Y: Fn(&O) -> bool,
{
    pub fn new(m: X, f: Y) -> Self {
        let p = PhantomData;
        Self { m, f, p }
    }
}

impl<I, O, X, Y> MapFilter<I, O> for MF<I, O, X, Y>
where
    X: Fn(I) -> O,
    Y: Fn(&O) -> bool,
{
    fn map_filter(&self, i: I) -> Option<O> {
        todo!()
    }

    type Compose<O3, X3, Y3>
        = MFMF<I, O, X, Y, O3, MF<O, O3, X3, Y3>>
    where
        X3: Fn(O) -> O3,
        Y3: Fn(&O3) -> bool;
    fn compose<O3, X3, Y3>(self, m: X3, f: Y3) -> Self::Compose<O3, X3, Y3>
    where
        X3: Fn(O) -> O3,
        Y3: Fn(&O3) -> bool,
    {
        MFMF::new(self.m, self.f, MF::new(m, f))
    }
}
