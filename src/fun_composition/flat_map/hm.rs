use crate::fun_composition::flat_map::{flat_map_queue::FlatMapQ, flat_map_trait::FlatMap};

pub struct Hm<H1: FlatMap, H2: FlatMapQ<I = H1::O>> {
    h1: H1,
    h2: H2,
}

impl<H1: FlatMap, H2: FlatMapQ<I = H1::O>> Hm<H1, H2> {
    pub fn new(h1: H1, h2: H2) -> Self {
        Self { h1, h2 }
    }
}

impl<H1: FlatMap, H2: FlatMapQ<I = H1::O>> FlatMap for Hm<H1, H2> {
    type I = H1::I;

    type O = H2::O;

    #[inline(always)]
    fn flat_map(&self, i: Self::I) -> impl IntoIterator<Item = Self::O> {
        self.h1
            .flat_map(i)
            .into_iter()
            .flat_map(|j| self.h2.flat_map(j))
    }
}
