use crate::xap::fun::map::MapFn;
use core::iter::FusedIterator;

pub struct MapI<I: Iterator, G: MapFn<I = I::Item>> {
    i: I,
    g: G,
}

impl<I: Iterator, G: MapFn<I = I::Item>> MapI<I, G> {
    pub fn new(i: I, g: G) -> Self {
        Self { i, g }
    }
}

impl<I: Iterator, G: MapFn<I = I::Item>> Iterator for MapI<I, G> {
    type Item = G::O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().map(|x| self.g.map(x))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.i.size_hint()
    }
}

impl<I: ExactSizeIterator, G: MapFn<I = I::Item>> ExactSizeIterator for MapI<I, G> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.i.len()
    }
}
impl<I: FusedIterator, G: MapFn<I = I::Item>> FusedIterator for MapI<I, G> {}
