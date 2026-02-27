use crate::generic_values::WhilstAtom;

pub struct WhilstVectorWhilstIter<I, T, W>
where
    I: Iterator<Item = WhilstAtom<T>>,
    W: Fn(&T) -> bool,
{
    iter: I,
    whilst: W,
}

impl<I, T, W> WhilstVectorWhilstIter<I, T, W>
where
    I: Iterator<Item = WhilstAtom<T>>,
    W: Fn(&T) -> bool,
{
    pub(crate) fn new(iter: I, whilst: W) -> Self {
        Self { iter, whilst }
    }
}

impl<I, T, W> Iterator for WhilstVectorWhilstIter<I, T, W>
where
    I: Iterator<Item = WhilstAtom<T>>,
    W: Fn(&T) -> bool,
{
    type Item = WhilstAtom<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| match x {
            WhilstAtom::Continue(x) if (self.whilst)(&x) => WhilstAtom::Continue(x),
            _ => WhilstAtom::Stop,
        })
    }
}
