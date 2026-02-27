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

    pub fn from_atom<T, Fm>(atom: WhilstAtom<T>, flat_map: Fm) -> Self
    where
        Fm: Fn(T) -> Vo,
    {
        let iter = match atom {
            WhilstAtom::Continue(x) => WhilstAtom::Continue(flat_map(x).into_iter()),
            WhilstAtom::Stop => WhilstAtom::Stop,
        };
        Self { iter }
    }

    pub fn u_from_atom<U, T, Fm>(u: *mut U, atom: WhilstAtom<T>, flat_map: Fm) -> Self
    where
        Fm: Fn(*mut U, T) -> Vo,
    {
        let iter = match atom {
            WhilstAtom::Continue(x) => WhilstAtom::Continue(flat_map(u, x).into_iter()),
            WhilstAtom::Stop => WhilstAtom::Stop,
        };
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
