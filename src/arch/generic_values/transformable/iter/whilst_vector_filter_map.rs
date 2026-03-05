use crate::generic_values::WhilstAtom;

pub struct WhilstVectorFilterMapIter<I, T, O, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(T) -> Option<O>,
{
    iter: I,
    filter_map: Fm,
}

impl<I, T, O, Fm> WhilstVectorFilterMapIter<I, T, O, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(T) -> Option<O>,
{
    pub(crate) fn new(iter: I, filter_map: Fm) -> Self {
        Self { iter, filter_map }
    }
}

impl<I, T, O, Fm> Iterator for WhilstVectorFilterMapIter<I, T, O, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(T) -> Option<O>,
{
    type Item = WhilstAtom<O>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find_map(|x| match x {
            WhilstAtom::Continue(x) => match (self.filter_map)(x) {
                Some(y) => Some(WhilstAtom::Continue(y)),
                None => None,
            },
            WhilstAtom::Stop => Some(WhilstAtom::Stop),
        })
    }
}
