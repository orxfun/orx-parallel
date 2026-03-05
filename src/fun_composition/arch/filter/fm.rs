use crate::fun_composition::filter::{filter_queue::FilterQ, filter_trait::Filter};

pub struct Fm<F1: Filter, F2: FilterQ<I = F1::I>> {
    f1: F1,
    f2: F2,
}

impl<F1: Filter, F2: FilterQ<I = F1::I>> Fm<F1, F2> {
    pub fn new(f1: F1, f2: F2) -> Self {
        Self { f1, f2 }
    }
}

impl<F1: Filter, F2: FilterQ<I = F1::I>> Filter for Fm<F1, F2> {
    type I = F1::I;

    #[inline(always)]
    fn filter(&self, i: &Self::I) -> bool {
        self.f1.filter(i) && self.f2.filter(i)
    }
}

impl<F1: Filter, F2: FilterQ<I = F1::I>> FilterQ for Fm<F1, F2> {
    type Compose<X>
        = Fm<F1, F2::Compose<X>>
    where
        X: Filter<I = Self::I>;

    fn compose<X>(self, x: X) -> Self::Compose<X>
    where
        X: Filter<I = Self::I>,
    {
        Fm::new(self.f1, self.f2.compose(x))
    }
}
