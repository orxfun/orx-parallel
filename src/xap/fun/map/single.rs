use crate::xap::fun::map::{r#fn::MapFn, pair::Mp, queue::MapQueue};

pub struct Ms<F: MapFn> {
    f: F,
}

impl<F: MapFn> Ms<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: MapFn> MapFn for Ms<F> {
    type I = F::I;

    type O = F::O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        self.f.map(i)
    }
}

impl<F: MapFn> MapQueue for Ms<F> {
    type Then<Q, H>
        = Mp<F, Ms<H>>
    where
        H: MapFn<I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: MapFn<I = Self::O, O = Q>,
    {
        Mp::new(self.f, Ms::new(h))
    }
}
