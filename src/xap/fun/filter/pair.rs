use crate::xap::fun::filter::{r#fn::FilterFn, queue::FilterQueue};

pub struct Fp<F: FilterFn, B: FilterQueue<I = F::I>> {
    f: F,
    b: B,
}

impl<F: FilterFn, B: FilterQueue<I = F::I>> Fp<F, B> {
    pub fn new(f: F, b: B) -> Self {
        Self { f, b }
    }
}

impl<F: FilterFn, B: FilterQueue<I = F::I>> FilterFn for Fp<F, B> {
    type I = F::I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        self.f.filter(i) && self.b.filter(i)
    }
}

impl<F: FilterFn, B: FilterQueue<I = F::I>> FilterQueue for Fp<F, B> {
    type Then<H>
        = Fp<F, B::Then<H>>
    where
        H: FilterFn<I = Self::I>;

    fn then<H>(self, h: H) -> Self::Then<H>
    where
        H: FilterFn<I = Self::I>,
    {
        Fp::new(self.f, self.b.then(h))
    }
}
