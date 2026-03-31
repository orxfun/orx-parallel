use crate::generic_values::{WhilstAtom, transformable::iter::WhilstAtomIter};

pub struct WhilstVectorFlatMapIter<I, T, Vo, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(T) -> Vo,
    Vo: IntoIterator,
{
    outer: I,
    inner: Option<WhilstAtomIter<Vo>>,
    flat_map: Fm,
}

impl<I, T, Vo, Fm> WhilstVectorFlatMapIter<I, T, Vo, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(T) -> Vo,
    Vo: IntoIterator,
{
    pub(crate) fn new(outer: I, flat_map: Fm) -> Self {
        Self {
            outer,
            inner: None,
            flat_map,
        }
    }

    fn next_inner(&mut self) -> Option<WhilstAtom<Vo::Item>> {
        debug_assert!(self.inner.is_none());
        match self.outer.next() {
            Some(x) => {
                let iter = match x {
                    WhilstAtom::Continue(x) => {
                        let iter = (self.flat_map)(x).into_iter();
                        WhilstAtom::Continue(iter)
                    }
                    WhilstAtom::Stop => WhilstAtom::Stop,
                };
                let inner = WhilstAtomIter::new(iter);
                self.inner = Some(inner);
                self.next()
            }
            None => None,
        }
    }
}

impl<I, T, Vo, Fm> Iterator for WhilstVectorFlatMapIter<I, T, Vo, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(T) -> Vo,
    Vo: IntoIterator,
{
    type Item = WhilstAtom<Vo::Item>;

    #[inline]
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
