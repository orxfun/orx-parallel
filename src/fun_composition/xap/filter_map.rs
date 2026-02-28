use crate::fun_composition::{
    fn_filter::FilterQ,
    fn_map::{MapQ, WrMap},
    xap::fn_xap::Xap,
};
use core::marker::PhantomData;

pub struct FilterMap<I, O, F, M>
where
    F: FilterQ<I = I>,
    M: MapQ<I = I, O = O>,
{
    f: F,
    m: M,
    phantom: PhantomData<I>,
}

impl<I, O, F, M> Xap<I> for FilterMap<I, O, F, M>
where
    F: FilterQ<I = I>,
    M: MapQ<I = I, O = O>,
{
    type O = Option<O>;

    fn run(&self, i: I) -> Self::O {
        match self.f.run(&i) {
            true => Some(self.m.run(i)),
            false => None,
        }
    }
}

impl<I, O, F, M> FilterMap<I, O, F, M>
where
    F: FilterQ<I = I>,
    M: MapQ<I = I, O = O>,
{
    fn new(f: F, m: M) -> Self {
        Self {
            f,
            m,
            phantom: PhantomData,
        }
    }

    pub fn map<M2, O2>(self, m2: M2) -> FilterMap<I, O2, F, M::Pb<WrMap<O, O2, M2>>>
    where
        M2: Fn(O) -> O2,
    {
        let m = self.m.push_back(WrMap::new(m2));
        FilterMap::new(self.f, m)
    }
}
