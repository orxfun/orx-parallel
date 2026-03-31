pub struct VectorUFlatMapIter<I, U, Fm, Vo>
where
    I: Iterator,
    Vo: IntoIterator,
    Fm: Fn(*mut U, I::Item) -> Vo,
{
    u: *mut U,
    outer: I,
    inner: Option<Vo::IntoIter>,
    flat_map: Fm,
}

impl<I, U, Fm, Vo> VectorUFlatMapIter<I, U, Fm, Vo>
where
    I: Iterator,
    Vo: IntoIterator,
    Fm: Fn(*mut U, I::Item) -> Vo,
{
    pub fn new(u: *mut U, outer: I, flat_map: Fm) -> Self {
        Self {
            u,
            outer,
            inner: None,
            flat_map,
        }
    }

    fn next_inner(&mut self) -> Option<Vo::Item> {
        debug_assert!(self.inner.is_none());
        match self.outer.next() {
            Some(x) => {
                let iter = (self.flat_map)(self.u, x);
                self.inner = Some(iter.into_iter());
                self.next()
            }
            None => None,
        }
    }
}

impl<I, U, Fm, Vo> Iterator for VectorUFlatMapIter<I, U, Fm, Vo>
where
    I: Iterator,
    Vo: IntoIterator,
    Fm: Fn(*mut U, I::Item) -> Vo,
{
    type Item = Vo::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            None => self.next_inner(),
            Some(inner) => match inner.next() {
                Some(x) => Some(x),
                None => self.next_inner(),
            },
        }
    }
}
