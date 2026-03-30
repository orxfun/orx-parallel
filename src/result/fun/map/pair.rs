use crate::result::fun::map::{fn_trait::MapRes, queue::MapResQueue};

#[derive(Clone, Copy)]
pub struct ResMp<F: MapRes, B: MapResQueue<I = F::O, E = F::E>> {
    f: F,
    b: B,
}

impl<F: MapRes, B: MapResQueue<I = F::O, E = F::E>> ResMp<F, B> {
    pub fn new(f: F, b: B) -> Self {
        Self { f, b }
    }
}

impl<F: MapRes, B: MapResQueue<I = F::O, E = F::E>> MapRes for ResMp<F, B> {
    type I = F::I;

    type O = B::O;

    type E = F::E;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Result<Self::O, Self::E> {
        match self.f.map(i) {
            Ok(x) => self.b.map(x),
            Err(e) => Err(e),
        }
    }
}

impl<F: MapRes, B: MapResQueue<I = F::O, E = F::E>> MapResQueue for ResMp<F, B> {
    type Then<Q, H>
        = ResMp<F, B::Then<Q, H>>
    where
        H: MapRes<E = Self::E, I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: MapRes<E = Self::E, I = Self::O, O = Q>,
    {
        ResMp::new(self.f, self.b.then(h))
    }
}
