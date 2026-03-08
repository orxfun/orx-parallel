use crate::xap::fun::map::Map;
use core::iter::FusedIterator;

pub struct MapIterMany<I: Iterator, G: Map<I = I::Item>> {
    i: I,
    g: G,
}

impl<I: Iterator, G: Map<I = I::Item>> MapIterMany<I, G> {
    pub fn new(i: I, g: G) -> Self {
        Self { i, g }
    }
}

impl<I: Iterator, G: Map<I = I::Item>> Iterator for MapIterMany<I, G> {
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

impl<I: ExactSizeIterator, G: Map<I = I::Item>> ExactSizeIterator for MapIterMany<I, G> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.i.len()
    }
}

impl<I: FusedIterator, G: Map<I = I::Item>> FusedIterator for MapIterMany<I, G> {}
