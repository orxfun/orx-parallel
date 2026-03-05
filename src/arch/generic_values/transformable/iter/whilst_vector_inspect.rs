use crate::generic_values::WhilstAtom;

pub struct WhilstVectorInspectIter<I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(&T),
{
    iter: I,
    inspect: F,
}

impl<I, T, F> WhilstVectorInspectIter<I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(&T),
{
    pub(crate) fn new(iter: I, inspect: F) -> Self {
        Self { iter, inspect }
    }
}

impl<I, T, F> Iterator for WhilstVectorInspectIter<I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(&T),
{
    type Item = WhilstAtom<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| match x {
            WhilstAtom::Continue(x) => {
                (self.inspect)(&x);
                WhilstAtom::Continue(x)
            }
            WhilstAtom::Stop => WhilstAtom::Stop,
        })
    }
}
