use crate::generic_values::whilst_atom::WhilstAtom;

pub struct WhilstAtomIter<Vo>
where
    Vo: IntoIterator,
{
    iter: WhilstAtom<Vo::IntoIter>,
}

impl<Vo> WhilstAtomIter<Vo>
where
    Vo: IntoIterator,
{
    pub(crate) fn new(iter: WhilstAtom<Vo::IntoIter>) -> Self {
        Self { iter }
    }
}

impl<Vo> Iterator for WhilstAtomIter<Vo>
where
    Vo: IntoIterator,
{
    type Item = WhilstAtom<Vo::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter {
            WhilstAtom::Continue(x) => x.next().map(WhilstAtom::Continue), // None if flat-map iterator is consumed
            WhilstAtom::Stop => Some(WhilstAtom::Stop),                    // input is Stop
        }
    }
}
