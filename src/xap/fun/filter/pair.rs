use crate::xap::fun::filter::{r#fn::FilterFn, queue::FilterQ};

pub struct FilterP<F: FilterFn, B: FilterQ<I = F::I>> {
    f: F,
    b: B,
}

impl<F: FilterFn, B: FilterQ<I = F::I>> FilterP<F, B> {
    pub fn new(f: F, b: B) -> Self {
        Self { f, b }
    }
}

impl<F: FilterFn, B: FilterQ<I = F::I>> FilterFn for FilterP<F, B> {
    type I = F::I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        self.f.filter(i) && self.b.filter(i)
    }
}

impl<F: FilterFn, B: FilterQ<I = F::I>> FilterQ for FilterP<F, B> {
    type Then<H>
        = FilterP<F, B::Then<H>>
    where
        H: FilterFn<I = Self::I>;

    fn then<H>(self, h: H) -> Self::Then<H>
    where
        H: FilterFn<I = Self::I>,
    {
        FilterP::new(self.f, self.b.then(h))
    }
}
