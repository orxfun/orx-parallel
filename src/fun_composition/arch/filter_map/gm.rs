use crate::fun_composition::filter_map::{
    filter_map_queue::FilterMapQ, filter_map_trait::FilterMap,
};

pub struct Gm<G1: FilterMap, G2: FilterMapQ<I = G1::O>> {
    g1: G1,
    g2: G2,
}

impl<G1: FilterMap, G2: FilterMapQ<I = G1::O>> Gm<G1, G2> {
    pub fn new(g1: G1, g2: G2) -> Self {
        Self { g1, g2 }
    }
}

impl<G1: FilterMap, G2: FilterMapQ<I = G1::O>> FilterMap for Gm<G1, G2> {
    type I = G1::I;

    type O = G2::O;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        match self.g1.filter_map(i) {
            Some(i) => self.g2.filter_map(i),
            None => None,
        }
    }
}

impl<G1: FilterMap, G2: FilterMapQ<I = G1::O>> FilterMapQ for Gm<G1, G2> {
    type Compose<X>
        = Gm<G1, G2::Compose<X>>
    where
        X: FilterMap<I = Self::O>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: FilterMap<I = Self::O>,
    {
        Gm::new(self.g1, self.g2.compose(x))
    }
}
