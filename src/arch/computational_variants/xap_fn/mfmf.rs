use crate::{
    computational_variants::xap_fn::{
        f0::F0, filter::Filter, m::M, map::Map, map_filter::MapFilter, xap::XapFn,
    },
    generic_values::Values,
};
use core::marker::PhantomData;

pub struct MFMF<I, O1, M1, F1, O2, Mf2>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
    Mf2: MapFilter<O1, O2>,
{
    m: M1,
    f: F1,
    mf2: Mf2,
    p: PhantomData<(I, O1, O2)>,
}

impl<I, O1, M1, F1, O2, Mf2> MFMF<I, O1, M1, F1, O2, Mf2>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
    Mf2: MapFilter<O1, O2>,
{
    pub fn new(m: M1, f: F1, mf2: Mf2) -> Self {
        let p = PhantomData;
        Self { m, f, mf2, p }
    }
}

impl<I, O1, M1, F1, O2, Mf2> MapFilter<I, O2> for MFMF<I, O1, M1, F1, O2, Mf2>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
    Mf2: MapFilter<O1, O2>,
{
    #[inline(always)]
    fn map_filter(&self, i: I) -> Option<O2> {
        let x = self.m.map(i);
        match self.f.filter(&x) {
            true => self.mf2.map_filter(x),
            false => None,
        }
    }

    type Compose<Q, M3, F3>
        = MFMF<I, O1, M1, F1, Q, Mf2::Compose<Q, M3, F3>>
    where
        M3: Map<O2, Q>,
        F3: Filter<Q>;

    fn compose<Q, M3, F3>(self, m: M3, f: F3) -> Self::Compose<Q, M3, F3>
    where
        M3: Map<O2, Q>,
        F3: Filter<Q>,
    {
        MFMF::new(self.m, self.f, self.mf2.compose(m, f))
    }

    type ComposeF<F3>
        = MFMF<I, O1, M1, F1, O2, Mf2::ComposeF<F3>>
    where
        F3: Fn(&O2) -> bool;

    fn compose_f<F3>(self, f: F3) -> Self::ComposeF<F3>
    where
        F3: Fn(&O2) -> bool,
    {
        MFMF::new(self.m, self.f, self.mf2.compose_f(f))
    }
}

impl<I, O1, M1, F1, O2, Mf2> XapFn<I, Option<O2>> for MFMF<I, O1, M1, F1, O2, Mf2>
where
    M1: Map<I, O1>,
    F1: Filter<O1>,
    Mf2: MapFilter<O1, O2>,
{
    fn xap(&self, i: I) -> Option<O2> {
        todo!()
    }

    type Map<Y, Q>
        = MFMF<I, O1, M1, F1, Q, Mf2::Compose<Q, M<O2, Q, Y>, F0<Q>>>
    where
        Y: Fn(O2) -> Q;

    fn map<Y, Q>(self, map: Y) -> Self::Map<Y, Q>
    where
        Y: Fn(O2) -> Q,
    {
        let m = M::new(map);
        let f = F0::<Q>::new();
        let mf2 = self.mf2.compose(m, f);
        MFMF::new(self.m, self.f, mf2)
    }

    type Filter<Y>
        = MFMF<I, O1, M1, F1, O2, Mf2::ComposeF<Y>>
    where
        Y: Fn(&O2) -> bool;

    fn filter<Y>(self, filter: Y) -> Self::Filter<Y>
    where
        Y: Fn(&O2) -> bool,
    {
        let mf2 = self.mf2.compose_f(filter);
        MFMF::new(self.m, self.f, mf2)
    }
}
