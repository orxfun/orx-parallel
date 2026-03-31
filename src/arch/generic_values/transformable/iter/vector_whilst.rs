use crate::generic_values::WhilstAtom;

pub struct VectorWhilstIter<I, W>
where
    I: Iterator,
    W: Fn(&I::Item) -> bool,
{
    iter: I,
    whilst: W,
}

impl<I, W> VectorWhilstIter<I, W>
where
    I: Iterator,
    W: Fn(&I::Item) -> bool,
{
    pub fn new(iter: I, whilst: W) -> Self {
        Self { iter, whilst }
    }
}

impl<I, W> Iterator for VectorWhilstIter<I, W>
where
    I: Iterator,
    W: Fn(&I::Item) -> bool,
{
    type Item = WhilstAtom<I::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| match (self.whilst)(&x) {
            true => WhilstAtom::Continue(x),
            false => WhilstAtom::Stop,
        })
    }
}
