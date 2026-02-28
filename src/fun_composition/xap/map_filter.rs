use crate::fun_composition::{
    fn_filter::{FilterQ, FnFilterUnit},
    fn_map::MapQ,
    xap::fn_xap::Xap,
};
use core::marker::PhantomData;

pub struct MapFilter<I, O, M, F>
where
    M: MapQ<I = I, O = O>,
    F: FilterQ<I = O>,
{
    m: M,
    f: F,
    phantom: PhantomData<I>,
}

impl<I, O, M, F> Xap<I> for MapFilter<I, O, M, F>
where
    M: MapQ<I = I, O = O>,
    F: FilterQ<I = O>,
{
    type O = Option<O>;

    fn run(&self, i: I) -> Self::O {
        let val = self.m.run(i);
        match self.f.run(&val) {
            true => Some(val),
            false => None,
        }
    }
}

impl<I, O, M, F> MapFilter<I, O, M, F>
where
    M: MapQ<I = I, O = O>,
    F: FilterQ<I = O>,
{
    fn new(m: M, f: F) -> Self {
        Self {
            m,
            f,
            phantom: PhantomData,
        }
    }

    pub fn filter<F2>(self, f2: F2) -> MapFilter<I, O, M, F::PushBack<FnFilterUnit<O, F2>>>
    where
        F2: Fn(&O) -> bool,
    {
        let f = self.f.push_back(FnFilterUnit::new(f2));
        MapFilter::new(self.m, f)
    }
}
