use crate::xap::fun::flat_map::FlatMap;

pub struct FlatMapIterMany<I: Iterator, G: FlatMap<I = I::Item>> {
    i: I,
    g: G,
    inner: Option<<G::O as IntoIterator>::IntoIter>,
}

impl<I: Iterator, G: FlatMap<I = I::Item>> FlatMapIterMany<I, G> {
    pub fn new(i: I, g: G) -> Self {
        let inner = None;
        Self { i, g, inner }
    }
}

impl<I: Iterator, G: FlatMap<I = I::Item>> Iterator for FlatMapIterMany<I, G> {
    type Item = <G::O as IntoIterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt;
            }

            match self.i.next() {
                Some(i) => self.inner = Some(self.g.flat_map(i).into_iter()),
                None => return None,
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            Some(inner) => (inner.size_hint().0, None),
            None => (0, None),
        }
    }
}

#[inline(always)]
fn and_then_or_clear<T, U>(opt: &mut Option<T>, f: impl FnOnce(&mut T) -> Option<U>) -> Option<U> {
    let x = f(opt.as_mut()?);
    if x.is_none() {
        *opt = None;
    }
    x
}
