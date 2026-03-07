use crate::xap::fun::filter::{r#fn::FilterFn, pair::Fp, queue::FilterQueue};

pub struct Fs<F: FilterFn> {
    f: F,
}

impl<F: FilterFn> Fs<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: FilterFn> FilterFn for Fs<F> {
    type I = F::I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        self.f.filter(i)
    }
}

impl<F: FilterFn> FilterQueue for Fs<F> {
    type Then<H>
        = Fp<F, Fs<H>>
    where
        H: FilterFn<I = Self::I>;

    fn then<H>(self, h: H) -> Self::Then<H>
    where
        H: FilterFn<I = Self::I>,
    {
        Fp::new(self.f, Fs::new(h))
    }
}
