use crate::computational_variants::xap_fn::map_filter::MapFilter;
use core::marker::PhantomData;

pub struct MFMF<I, O1, X1, Y1, O2, Mf2>
where
    X1: Fn(I) -> O1,
    Y1: Fn(&O1) -> bool,
    Mf2: MapFilter<O1, O2>,
{
    m1: X1,
    f1: Y1,
    mf2: Mf2,
    p: PhantomData<(I, O2)>,
}

impl<I, O1, X1, Y1, O2, Mf2> MFMF<I, O1, X1, Y1, O2, Mf2>
where
    X1: Fn(I) -> O1,
    Y1: Fn(&O1) -> bool,
    Mf2: MapFilter<O1, O2>,
{
    pub fn new(m1: X1, f1: Y1, mf2: Mf2) -> Self {
        let p = PhantomData;
        Self { m1, f1, mf2, p }
    }
}

impl<I, O1, X1, Y1, O2, Mf2> MapFilter<I, O2> for MFMF<I, O1, X1, Y1, O2, Mf2>
where
    X1: Fn(I) -> O1,
    Y1: Fn(&O1) -> bool,
    Mf2: MapFilter<O1, O2>,
{
    #[inline(always)]
    fn map_filter(&self, i: I) -> Option<O2> {
        let x1 = (self.m1)(i);
        match (self.f1)(&x1) {
            true => self.mf2.map_filter(x1),
            false => None,
        }
    }

    // type Compose<Z, Q>
    //     = MFMF<I, O1, X1, Y1, Q, Mf2::Compose<Z, Q>>
    // where
    //     Z: MapFilter<O2, Q>;

    // fn compose<Z, Q>(self, z: Z) -> Self::Compose<Z, Q>
    // where
    //     Z: MapFilter<O2, Q>,
    // {
    //     MFMF::new(self.m1, self.f1, self.mf2.compose(z))
    // }
}
