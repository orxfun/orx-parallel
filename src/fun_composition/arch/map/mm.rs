use crate::fun_composition::map::{map_queue::MapQ, map_trait::Map};

pub struct Mm<M1: Map, M2: MapQ<I = M1::O>> {
    m1: M1,
    m2: M2,
}

impl<M1: Map, M2: MapQ<I = M1::O>> Mm<M1, M2> {
    pub fn new(m1: M1, m2: M2) -> Self {
        Self { m1, m2 }
    }
}

impl<M1: Map, M2: MapQ<I = M1::O>> Map for Mm<M1, M2> {
    type I = M1::I;

    type O = M2::O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        self.m2.map(self.m1.map(i))
    }
}

impl<M1: Map, M2: MapQ<I = M1::O>> MapQ for Mm<M1, M2> {
    type Compose<X>
        = Mm<M1, M2::Compose<X>>
    where
        X: Map<I = Self::O>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: Map<I = Self::O>,
    {
        Mm::new(self.m1, self.m2.compose(x))
    }
}
