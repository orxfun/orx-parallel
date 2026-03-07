use crate::xap::fun::filter::r#fn::FilterFn;
use core::iter::FusedIterator;

pub struct FilterI<I: Iterator, G: FilterFn<I = I::Item>> {
    i: I,
    g: G,
}

impl<I: Iterator, G: FilterFn<I = I::Item>> FilterI<I, G> {
    pub fn new(i: I, g: G) -> Self {
        Self { i, g }
    }
}

impl<I: Iterator, G: FilterFn<I = I::Item>> Iterator for FilterI<I, G> {
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.i.find_map(|x| self.g.filter(&x).then_some(x))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.i.size_hint()
    }
}

impl<I: ExactSizeIterator, G: FilterFn<I = I::Item>> ExactSizeIterator for FilterI<I, G> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.i.len()
    }
}

impl<I: FusedIterator, G: FilterFn<I = I::Item>> FusedIterator for FilterI<I, G> {}
