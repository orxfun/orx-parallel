use core::marker::PhantomData;

use crate::computational_variants::xap_fn::{map_filter::MapFilter, mfmf::MFMF};

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
}
