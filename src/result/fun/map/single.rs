use crate::result::fun::map::{fn_trait::MapRes, pair::ResMp, queue::MapResQueue};

#[derive(Clone, Copy)]
pub struct Ms<F: MapRes> {
    f: F,
}

impl<F: MapRes> Ms<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: MapRes> MapRes for Ms<F> {
    type I = F::I;

    type O = F::O;

    type E = F::E;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Result<Self::O, Self::E> {
        self.f.map(i)
    }
}

impl<F: MapRes> MapResQueue for Ms<F> {
    type Then<Q, H>
        = ResMp<F, Ms<H>>
    where
        H: MapRes<E = Self::E, I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: MapRes<E = Self::E, I = Self::O, O = Q>,
    {
        ResMp::new(self.f, Ms::new(h))
    }
}
