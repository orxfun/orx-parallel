use crate::xap::fun::filter::{fn_trait::Filter, pair::Fp, queue::FilterQueue};

pub struct Fs<F: Filter> {
    f: F,
}

impl<F: Filter> Fs<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Filter> Filter for Fs<F> {
    type I = F::I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        self.f.filter(i)
    }
}

impl<F: Filter> FilterQueue for Fs<F> {
    type Then<H>
        = Fp<F, Fs<H>>
    where
        H: Filter<I = Self::I>;

    fn then<H>(self, h: H) -> Self::Then<H>
    where
        H: Filter<I = Self::I>,
    {
        Fp::new(self.f, Fs::new(h))
    }
}
