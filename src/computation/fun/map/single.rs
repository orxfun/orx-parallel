#[derive(Clone, Copy)]
pub struct Ms<F: Map> {
    f: F,
}

impl<F: Map> Ms<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Map> Map for Ms<F> {
    type I = F::I;

    type O = F::O;

    #[inline(always)]
    fn map(&self, i: Self::I) -> Self::O {
        self.f.map(i)
    }
}

impl<F: Map> MapQueue for Ms<F> {
    type Then<Q, H>
        = Mp<F, Ms<H>>
    where
        H: Map<I = Self::O, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        H: Map<I = Self::O, O = Q>,
    {
        Mp::new(self.f, Ms::new(h))
    }
}
