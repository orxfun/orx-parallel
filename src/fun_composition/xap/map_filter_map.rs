use crate::fun_composition::{
    fn_filter::{FilterQ, FilterQSingle, WrFilter},
    fn_map::{MapQ, WrMap},
    xap::{
        fn_xap::Xap,
        map_filter::{MapFilterQ, MapFilterQSingle, Mf},
    },
};
use core::marker::PhantomData;

pub struct Mfm<I, O1, M1, F, O2, M2>
where
    M1: MapQ<I = I, O = O1>,
    F: FilterQ<I = O1>,
    M2: MapQ<I = O1, O = O2>,
{
    m1: M1,
    m2: M2,
    f: F,
    phantom: PhantomData<I>,
}

impl<I, O1, M1, F, O2, M2> Xap<I> for Mfm<I, O1, M1, F, O2, M2>
where
    M1: MapQ<I = I, O = O1>,
    F: FilterQ<I = O1>,
    M2: MapQ<I = O1, O = O2>,
{
    type O = Option<O2>;

    fn run(&self, i: I) -> Self::O {
        let x = self.m1.run(i);
        match self.f.run(&x) {
            true => Some(self.m2.run(x)),
            false => None,
        }
    }
}

impl<I, O1, M1, F, O2, M2> Mfm<I, O1, M1, F, O2, M2>
where
    M1: MapQ<I = I, O = O1>,
    F: FilterQ<I = O1>,
    M2: MapQ<I = O1, O = O2>,
{
    fn new(m1: M1, f: F, m2: M2) -> Self {
        Self {
            m1,
            m2,
            f,
            phantom: PhantomData,
        }
    }

    pub fn map<O3, M3>(self, f: M3) -> Mfm<I, O1, M1, F, O3, M2::Pb<WrMap<O2, O3, M3>>>
    where
        M3: Fn(O2) -> O3,
    {
        let m2 = self.m2.push_back(WrMap::new(f));
        Mfm::new(self.m1, self.f, m2)
    }

    pub fn filter<F2>(self, f: F2)
    where
        F2: Fn(&O2) -> bool,
    {
        let mf1 = Mf::new(self.m1, self.f);
        let mf2 = Mf::new(self.m2, FilterQSingle::from(WrFilter::new(f)));

        // le1t x = MapFilterQSingle::from(mf1).push_back(mf2);
        //
    }
}
