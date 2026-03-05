use crate::fun_composition::map::{map_queue::MapQ, map_trait::Map, mm::Mm};

pub struct Ms<M: Map> {
    m: M,
}

impl<M: Map> Ms<M> {
    pub fn new(m: M) -> Self {
        Self { m }
    }
}

impl<M: Map> Map for Ms<M> {
    type I = M::I;

    type O = M::O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        self.m.map(i)
    }
}

impl<M: Map> MapQ for Ms<M> {
    type Compose<X>
        = Mm<M, Ms<X>>
    where
        X: Map<I = Self::O>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: Map<I = Self::O>,
    {
        Mm::new(self.m, Ms::new(x))
    }
}
