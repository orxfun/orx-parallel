use crate::generic_values::WhilstAtom;

pub struct WhilstVectorFilterIter<I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(&T) -> bool,
{
    iter: I,
    filter: F,
}

impl<I, T, F> WhilstVectorFilterIter<I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(&T) -> bool,
{
    pub(crate) fn new(iter: I, filter: F) -> Self {
        Self { iter, filter }
    }
}

impl<I, T, F> Iterator for WhilstVectorFilterIter<I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(&T) -> bool,
{
    type Item = WhilstAtom<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find(|x| match x {
            WhilstAtom::Continue(x) => (self.filter)(x),
            WhilstAtom::Stop => true,
        })
    }
}
