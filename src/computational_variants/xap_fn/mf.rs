use crate::computational_variants::xap_fn::{
    filter::Filter, map::Map, map_filter::MapFilter, mfmf::MFMF,
};
use core::marker::PhantomData;

pub struct MF<I, O1, M1, F1>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
{
    m: M1,
    f: F1,
    p: PhantomData<(I, O1)>,
}

impl<I, O1, M1, F1> MF<I, O1, M1, F1>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
{
    pub fn new(m: M1, f: F1) -> Self {
        let p = PhantomData;
        Self { m, f, p }
    }
}

impl<I, O1, M1, F1> MapFilter<I, O1> for MF<I, O1, M1, F1>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
{
    #[inline(always)]
    fn map_filter(&self, i: I) -> Option<O1> {
        let x = self.m.map(i);
        match self.f.filter(&x) {
            true => Some(x),
            false => None,
        }
    }

    type Compose<Q, M3, F3>
        = MFMF<I, O1, M1, F1, Q, MF<O1, Q, M3, F3>>
    where
        M3: Map<O1, Q>,
        F3: Filter<Q>;

    fn compose<Q, M3, F3>(self, m: M3, f: F3) -> Self::Compose<Q, M3, F3>
    where
        M3: Map<O1, Q>,
        F3: Filter<Q>,
    {
        MFMF::new(self.m, self.f, MF::new(m, f))
    }

    type ComposeF<F3>
        = MF<I, O1, M1, F1::Compose<F3>>
    where
        F3: Fn(&O1) -> bool;

    fn compose_f<F3>(self, f: F3) -> Self::ComposeF<F3>
    where
        F3: Fn(&O1) -> bool,
    {
        let f = self.f.compose(f);
        MF::new(self.m, f)
    }
}
