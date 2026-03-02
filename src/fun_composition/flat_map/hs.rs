use crate::fun_composition::flat_map::flat_map_trait::FlatMap;

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
