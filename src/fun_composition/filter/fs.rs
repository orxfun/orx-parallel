use crate::fun_composition::filter::{filter_queue::FilterQ, filter_trait::Filter, fm::Fm};

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

impl<F: Filter> FilterQ for Fs<F> {
    type Compose<X>
        = Fm<F, Fs<X>>
    where
        X: Filter<I = Self::I>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: Filter<I = Self::I>,
    {
        Fm::new(self.f, Fs::new(x))
    }
}
