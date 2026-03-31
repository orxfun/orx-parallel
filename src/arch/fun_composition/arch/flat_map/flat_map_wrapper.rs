use crate::fun_composition::flat_map::flat_map_trait::FlatMap;
use core::marker::PhantomData;

pub struct FnFlatMap<I, Vo: IntoIterator, H: Fn(I) -> Vo> {
    h: H,
    p: PhantomData<(I, Vo)>,
}

impl<I, Vo: IntoIterator, H: Fn(I) -> Vo> FnFlatMap<I, Vo, H> {
    pub fn new(h: H) -> Self {
        let p = PhantomData;
        Self { h, p }
    }
}

impl<I, Vo: IntoIterator, H: Fn(I) -> Vo> FlatMap for FnFlatMap<I, Vo, H> {
    type I = I;

    type O = Vo::Item;

    #[inline(always)]
    fn flat_map(&self, i: Self::I) -> impl IntoIterator<Item = Self::O> {
        (self.h)(i)
    }
}
