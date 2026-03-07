use crate::xap::fun::filter::{r#fn::FilterFn, pair::FilterP, queue::FilterQ};

pub struct FilterS<F: FilterFn> {
    f: F,
}

impl<F: FilterFn> FilterS<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: FilterFn> FilterFn for FilterS<F> {
    type I = F::I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        self.f.filter(i)
    }
}

impl<F: FilterFn> FilterQ for FilterS<F> {
    type Then<H>
        = FilterP<F, FilterS<H>>
    where
        H: FilterFn<I = Self::I>;

    fn then<H>(self, h: H) -> Self::Then<H>
    where
        H: FilterFn<I = Self::I>,
    {
        FilterP::new(self.f, FilterS::new(h))
    }
}
