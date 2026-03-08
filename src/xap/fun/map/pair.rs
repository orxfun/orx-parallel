use crate::xap::fun::map::{fn_trait::Map, queue::MapQueue};

pub struct Mp<F: Map, B: MapQueue<I = F::O>> {
    f: F,
    b: B,
}

impl<F: Map, B: MapQueue<I = F::O>> Mp<F, B> {
    pub fn new(f: F, b: B) -> Self {
        Self { f, b }
    }
}

impl<F: Map, B: MapQueue<I = F::O>> Map for Mp<F, B> {
    type I = F::I;

    type O = B::O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        self.b.map(self.f.map(i))
    }
}

impl<F: Map, B: MapQueue<I = F::O>> MapQueue for Mp<F, B> {
    type Then<Q, H>
        = Mp<F, B::Then<Q, H>>
    where
        H: Map<I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: Map<I = Self::O, O = Q>,
    {
        Mp::new(self.f, self.b.then(h))
    }
}
