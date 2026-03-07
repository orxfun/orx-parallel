use crate::xap::fun::map::{r#fn::MapFn, queue::MapQ};

pub struct MapP<F: MapFn, B: MapQ<I = F::O>> {
    f: F,
    b: B,
}

impl<F: MapFn, B: MapQ<I = F::O>> MapP<F, B> {
    pub fn new(f: F, b: B) -> Self {
        Self { f, b }
    }
}

impl<F: MapFn, B: MapQ<I = F::O>> MapFn for MapP<F, B> {
    type I = F::I;

    type O = B::O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        self.b.map(self.f.map(i))
    }
}

impl<F: MapFn, B: MapQ<I = F::O>> MapQ for MapP<F, B> {
    type Then<Q, H>
        = MapP<F, B::Then<Q, H>>
    where
        H: MapFn<I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: MapFn<I = Self::O, O = Q>,
    {
        MapP::new(self.f, self.b.then(h))
    }
}
