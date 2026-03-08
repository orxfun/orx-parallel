use crate::xap::fun::flat_map::FlatMapFn;

pub struct FlatMapIterMany<I: Iterator, G: FlatMapFn<I = I::Item>> {
    i: I,
    g: G,
    inner: Option<<G::O as IntoIterator>::IntoIter>,
}

impl<I: Iterator, G: FlatMapFn<I = I::Item>> FlatMapIterMany<I, G> {
    pub fn new(i: I, g: G) -> Self {
        let inner = None;
        Self { i, g, inner }
    }

    #[inline]
    pub fn next_from_next_iter(&mut self) -> Option<<G::O as IntoIterator>::Item> {
        self.i.next().and_then(|i| {
            self.inner = Some(self.g.flat_map(i).into_iter());
            self.next()
        })
    }
}

impl<I: Iterator, G: FlatMapFn<I = I::Item>> Iterator for FlatMapIterMany<I, G> {
    type Item = <G::O as IntoIterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            Some(x) => {
                let next = x.next();
                match next.is_some() {
                    true => next,
                    false => self.next_from_next_iter(),
                }
            }
            None => self.next_from_next_iter(),
        }
    }
}
