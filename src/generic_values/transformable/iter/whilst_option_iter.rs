use crate::generic_values::{WhilstAtom, WhilstOption};

pub struct WhilstOptionIter<I>
where
    I: Iterator,
{
    iter: WhilstOption<I>,
}

impl<I> WhilstOptionIter<I>
where
    I: Iterator,
{
    pub fn new(iter: WhilstOption<I>) -> Self {
        Self { iter }
    }
}

impl<I> Iterator for WhilstOptionIter<I>
where
    I: Iterator,
{
    type Item = WhilstAtom<I::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter {
            WhilstOption::ContinueSome(x) => x.next().map(WhilstAtom::Continue), // None if iterator is consumed
            WhilstOption::ContinueNone => None, // iterator is created on None => empty iterator
            WhilstOption::Stop => Some(WhilstAtom::Stop), // input iterator is Stop
        }
    }
}
