use crate::xap::fun::filter_map::FilterMap;
use core::iter::FusedIterator;

pub struct FilterMapIterMany<I: Iterator, G: FilterMap<I = I::Item>> {
    i: I,
    g: G,
}

impl<I: Iterator, G: FilterMap<I = I::Item>> FilterMapIterMany<I, G> {
    pub fn new(i: I, g: G) -> Self {
        Self { i, g }
    }
}

impl<I: Iterator, G: FilterMap<I = I::Item>> Iterator for FilterMapIterMany<I, G> {
    type Item = G::O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // self.i.next().map(|x| self.g.map(x))
        loop {
            match self.i.next() {
                Some(x) => {
                    let y = self.g.filter_map(x);
                    if y.is_some() {
                        return y;
                    }
                }
                None => return None,
            }
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.i.size_hint()
    }
}

impl<I: ExactSizeIterator, G: FilterMap<I = I::Item>> ExactSizeIterator
    for FilterMapIterMany<I, G>
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.i.len()
    }
}

impl<I: FusedIterator, G: FilterMap<I = I::Item>> FusedIterator for FilterMapIterMany<I, G> {}
