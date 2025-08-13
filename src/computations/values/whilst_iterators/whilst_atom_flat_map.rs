use crate::computations::values::whilst_atom::WhilstAtom;

pub struct WhilstAtomFlatMapIter<Vo>
where
    Vo: IntoIterator,
{
    current_iter: WhilstAtom<Vo::IntoIter>,
}

impl<Vo> WhilstAtomFlatMapIter<Vo>
where
    Vo: IntoIterator,
{
    pub fn from_atom<T, Fm>(atom: WhilstAtom<T>, flat_map: Fm) -> Self
    where
        Fm: Fn(T) -> Vo,
    {
        let current_iter = match atom {
            WhilstAtom::Continue(x) => WhilstAtom::Continue(flat_map(x).into_iter()),
            WhilstAtom::Stop => WhilstAtom::Stop,
        };
        Self { current_iter }
    }
}

impl<Vo> Iterator for WhilstAtomFlatMapIter<Vo>
where
    Vo: IntoIterator,
{
    type Item = WhilstAtom<Vo::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.current_iter {
            WhilstAtom::Continue(x) => match x.next() {
                Some(x) => Some(WhilstAtom::Continue(x)),
                None => None, // flat map iterator is consumed
            },
            WhilstAtom::Stop => Some(WhilstAtom::Stop), // input is Stop
        }
    }
}
