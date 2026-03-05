use crate::fun_composition::filter_map::{
    filter_map_queue::FilterMapQ, filter_map_trait::FilterMap, gm::Gm,
};

pub struct Gs<G: FilterMap> {
    g: G,
}

impl<G: FilterMap> Gs<G> {
    pub fn new(g: G) -> Self {
        Self { g }
    }
}

impl<G: FilterMap> FilterMap for Gs<G> {
    type I = G::I;

    type O = G::O;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        self.g.filter_map(i)
    }
}

impl<G: FilterMap> FilterMapQ for Gs<G> {
    type Compose<X>
        = Gm<G, Gs<X>>
    where
        X: FilterMap<I = Self::O>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: FilterMap<I = Self::O>,
    {
        Gm::new(self.g, Gs::new(x))
    }
}
