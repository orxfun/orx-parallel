use crate::xap::count::iter::FlatMapIterMany;
use crate::xap::fun::flat_map::{FlatMap, queue::FlatMapQueue};

#[derive(Clone, Copy)]
pub struct FMp<F: FlatMap, B: FlatMapQueue<I = <F::O as IntoIterator>::Item>> {
    f: F,
    b: B,
}

impl<F: FlatMap, B: FlatMapQueue<I = <F::O as IntoIterator>::Item>> FMp<F, B> {
    pub fn new(f: F, b: B) -> Self {
        Self { f, b }
    }
}

impl<F: FlatMap, B: FlatMapQueue<I = <F::O as IntoIterator>::Item>> FlatMap for FMp<F, B> {
    type I = F::I;

    type O = FlatMapIterMany<<F::O as IntoIterator>::IntoIter, B>;

    #[inline]
    fn flat_map(&self, i: Self::I) -> Self::O {
        let iter = self.f.flat_map(i).into_iter();
        FlatMapIterMany::new(iter, self.b)
    }
}

impl<F: FlatMap, B: FlatMapQueue<I = <F::O as IntoIterator>::Item>> FlatMapQueue for FMp<F, B> {
    type Then<Q, H>
        = FMp<F, B::Then<Q, H>>
    where
        Q: IntoIterator,
        H: FlatMap<I = <Self::O as IntoIterator>::Item, O = Q>;

    fn then<Q, H>(self, h: H) -> Self::Then<Q, H>
    where
        Q: IntoIterator,
        H: FlatMap<I = <Self::O as IntoIterator>::Item, O = Q>,
    {
        FMp::new(self.f, self.b.then(h))
    }
}
