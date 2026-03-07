use crate::xap::fun::map::{r#fn::MapFn, pair::MapP, queue::MapQ};

pub struct MapS<F: MapFn> {
    f: F,
}

impl<F: MapFn> MapS<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: MapFn> MapFn for MapS<F> {
    type I = F::I;

    type O = F::O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        self.f.map(i)
    }
}

impl<F: MapFn> MapQ for MapS<F> {
    type Then<Q, H>
        = MapP<F, MapS<H>>
    where
        H: MapFn<I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: MapFn<I = Self::O, O = Q>,
    {
        MapP::new(self.f, MapS::new(h))
    }
}
