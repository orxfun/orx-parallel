use crate::infallible::fun::map::Map;
use crate::result_arch::fun::map::{fn_trait::MapRes, queue::MapResQueue};

#[derive(Clone, Copy)]
pub struct ResMp<F: MapRes, B: MapResQueue<E = F::E, I = F::O>> {
    f: F,
    b: B,
}

impl<F: MapRes, B: MapResQueue<E = F::E, I = F::O>> ResMp<F, B> {
    pub fn new(f: F, b: B) -> Self {
        Self { f, b }
    }
}

impl<F: MapRes, B: MapResQueue<E = F::E, I = F::O>> MapRes for ResMp<F, B> {
    type I = F::I;

    type O = B::O;

    type E = F::E;

    #[inline(always)]
    fn map_res(&self, i: Self::I) -> Result<Self::O, Self::E> {
        self.f.map_res(i).and_then(|x| self.b.map_res(x))
    }
}

impl<F: MapRes, B: MapResQueue<E = F::E, I = F::O>> MapResQueue for ResMp<F, B> {
    type Then<Q, H>
        = ResMp<F, B::Then<Q, H>>
    where
        H: Map<I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: Map<I = Self::O, O = Q>,
    {
        ResMp::new(self.f, self.b.then(h))
    }
}
