use crate::fun_composition::flat_map::{flat_map_queue::FlatMapQ, flat_map_trait::FlatMap, hm::Hm};

pub struct Hs<H: FlatMap> {
    h: H,
}

impl<H: FlatMap> Hs<H> {
    pub fn new(h: H) -> Self {
        Self { h }
    }
}

impl<H: FlatMap> FlatMap for Hs<H> {
    type I = H::I;

    type O = H::O;

    #[inline(always)]
    fn flat_map(&self, i: Self::I) -> impl IntoIterator<Item = Self::O> {
        self.h.flat_map(i)
    }
}

impl<H: FlatMap> FlatMapQ for Hs<H> {
    type Compose<X>
        = Hm<H, Hs<X>>
    where
        X: FlatMap<I = Self::O>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: FlatMap<I = Self::O>,
    {
        Hm::new(self.h, Hs::new(x))
    }
}
