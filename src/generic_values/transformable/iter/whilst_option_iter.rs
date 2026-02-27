use crate::generic_values::{WhilstAtom, WhilstOption};

pub struct WhilstOptionIter<Vo>
where
    Vo: IntoIterator,
{
    iter: WhilstOption<Vo::IntoIter>,
}

impl<Vo> WhilstOptionIter<Vo>
where
    Vo: IntoIterator,
{
    pub fn from_option<T, Fm>(atom: WhilstOption<T>, flat_map: Fm) -> Self
    where
        Fm: Fn(T) -> Vo,
    {
        let iter = match atom {
            WhilstOption::ContinueSome(x) => WhilstOption::ContinueSome(flat_map(x).into_iter()),
            WhilstOption::ContinueNone => WhilstOption::ContinueNone,
            WhilstOption::Stop => WhilstOption::Stop,
        };
        Self { iter }
    }

    pub fn u_from_option<U, T, Fm>(u: *mut U, atom: WhilstOption<T>, flat_map: Fm) -> Self
    where
        Fm: Fn(*mut U, T) -> Vo,
    {
        let iter = match atom {
            WhilstOption::ContinueSome(x) => WhilstOption::ContinueSome(flat_map(u, x).into_iter()),
            WhilstOption::ContinueNone => WhilstOption::ContinueNone,
            WhilstOption::Stop => WhilstOption::Stop,
        };
        Self { iter }
    }
}

impl<Vo> Iterator for WhilstOptionIter<Vo>
where
    Vo: IntoIterator,
{
    type Item = WhilstAtom<Vo::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter {
            WhilstOption::ContinueSome(x) => x.next().map(WhilstAtom::Continue), // None if iterator is consumed
            WhilstOption::ContinueNone => None, // iterator is created on None => empty iterator
            WhilstOption::Stop => Some(WhilstAtom::Stop), // input iterator is Stop
        }
    }
}
