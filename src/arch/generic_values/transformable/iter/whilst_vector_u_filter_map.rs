use crate::generic_values::WhilstAtom;

pub struct WhilstVectorUFilterMapIter<U, I, T, O, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(*mut U, T) -> Option<O>,
{
    u: *mut U,
    iter: I,
    filter_map: Fm,
}

impl<U, I, T, O, Fm> WhilstVectorUFilterMapIter<U, I, T, O, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(*mut U, T) -> Option<O>,
{
    pub(crate) fn new(u: *mut U, iter: I, filter_map: Fm) -> Self {
        Self {
            u,
            iter,
            filter_map,
        }
    }
}

impl<U, I, T, O, Fm> Iterator for WhilstVectorUFilterMapIter<U, I, T, O, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(*mut U, T) -> Option<O>,
{
    type Item = WhilstAtom<O>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find_map(|x| match x {
            WhilstAtom::Continue(x) => match (self.filter_map)(self.u, x) {
                Some(y) => Some(WhilstAtom::Continue(y)),
                None => None,
            },
            WhilstAtom::Stop => Some(WhilstAtom::Stop),
        })
    }
}
