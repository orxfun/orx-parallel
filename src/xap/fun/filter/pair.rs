use crate::xap::fun::filter::{fn_trait::Filter, queue::FilterQueue};

pub struct Fp<F: Filter, B: FilterQueue<I = F::I>> {
    f: F,
    b: B,
}

impl<F: Filter, B: FilterQueue<I = F::I>> Fp<F, B> {
    pub fn new(f: F, b: B) -> Self {
        Self { f, b }
    }
}

impl<F: Filter, B: FilterQueue<I = F::I>> Filter for Fp<F, B> {
    type I = F::I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        self.f.filter(i) && self.b.filter(i)
    }
}

impl<F: Filter, B: FilterQueue<I = F::I>> FilterQueue for Fp<F, B> {
    type Then<H>
        = Fp<F, B::Then<H>>
    where
        H: Filter<I = Self::I>;

    fn then<H>(self, h: H) -> Self::Then<H>
    where
        H: Filter<I = Self::I>,
    {
        Fp::new(self.f, self.b.then(h))
    }
}
