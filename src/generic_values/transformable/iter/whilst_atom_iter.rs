use crate::generic_values::whilst_atom::WhilstAtom;

pub struct WhilstAtomIter<I>
where
    I: IntoIterator,
{
    iter: WhilstAtom<I::IntoIter>,
}

impl<I> WhilstAtomIter<I>
where
    I: IntoIterator,
{
    pub fn new(iter: WhilstAtom<I::IntoIter>) -> Self {
        Self { iter }
    }
}

impl<I> Iterator for WhilstAtomIter<I>
where
    I: IntoIterator,
{
    type Item = WhilstAtom<I::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter {
            WhilstAtom::Continue(x) => x.next().map(WhilstAtom::Continue), // None if iterator is consumed
            WhilstAtom::Stop => Some(WhilstAtom::Stop), // Stop if input iter is Stop
        }
    }
}
