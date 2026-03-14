use crate::xap::fun::flat_map::{FlatMap, pair::FMp, queue::FlatMapQueue};

#[derive(Clone, Copy)]
pub struct FMs<F: FlatMap> {
    f: F,
}

impl<F: FlatMap> FMs<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: FlatMap> FlatMap for FMs<F> {
    type I = F::I;

    type O = F::O;

    #[inline]
    fn flat_map(&self, i: Self::I) -> Self::O {
        self.f.flat_map(i)
    }
}

impl<F: FlatMap> FlatMapQueue for FMs<F> {
    type Then<Q, H>
        = FMp<F, FMs<H>>
    where
        Q: IntoIterator,
        H: FlatMap<I = <F::O as IntoIterator>::Item, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        Q: IntoIterator,
        H: FlatMap<I = <Self::O as IntoIterator>::Item, O = Q>,
    {
        FMp::new(self.f, FMs::new(h))
    }
}
