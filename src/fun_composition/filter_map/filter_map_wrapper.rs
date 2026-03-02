use core::marker::PhantomData;

use crate::fun_composition::filter_map::filter_map_trait::FilterMap;

pub struct FnFilterMap<I, O, G: Fn(I) -> Option<O>> {
    g: G,
    p: PhantomData<(I, O)>,
}

impl<I, O, G: Fn(I) -> Option<O>> FnFilterMap<I, O, G> {
    pub fn new(g: G) -> Self {
        let p = PhantomData;
        Self { g, p }
    }
}

impl<I, O, G: Fn(I) -> Option<O>> FilterMap for FnFilterMap<I, O, G> {
    type I = I;

    type O = O;

    #[inline(always)]
    fn filter_map(&self, i: Self::I) -> Option<Self::O> {
        (self.g)(i)
    }
}
