use crate::generic_values::WhilstAtom;

pub struct WhilstVectorUFilterIter<U, I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(*mut U, &T) -> bool,
{
    u: *mut U,
    iter: I,
    filter: F,
}

impl<U, I, T, F> WhilstVectorUFilterIter<U, I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(*mut U, &T) -> bool,
{
    pub(crate) fn new(u: *mut U, iter: I, filter: F) -> Self {
        Self { u, iter, filter }
    }
}

impl<U, I, T, F> Iterator for WhilstVectorUFilterIter<U, I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(*mut U, &T) -> bool,
{
    type Item = WhilstAtom<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find(|x| match x {
            WhilstAtom::Continue(x) => (self.filter)(self.u, x),
            WhilstAtom::Stop => true,
        })
    }
}
