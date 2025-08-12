use crate::computations::{WhilstAtom, generalized_values::whilst_option::WhilstOption};

pub struct WhilstOptionFlatMapIter<Vo>
where
    Vo: IntoIterator,
{
    current_iter: WhilstOption<Vo::IntoIter>,
}

impl<Vo> WhilstOptionFlatMapIter<Vo>
where
    Vo: IntoIterator,
{
    pub fn from_option<T, Fm>(atom: WhilstOption<T>, flat_map: Fm) -> Self
    where
        Fm: Fn(T) -> Vo,
    {
        let current_iter = match atom {
            WhilstOption::ContinueSome(x) => WhilstOption::ContinueSome(flat_map(x).into_iter()),
            WhilstOption::ContinueNone => WhilstOption::ContinueNone,
            WhilstOption::Stop => WhilstOption::Stop,
        };
        Self { current_iter }
    }
}

impl<Vo> Iterator for WhilstOptionFlatMapIter<Vo>
where
    Vo: IntoIterator,
{
    type Item = WhilstAtom<Vo::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.current_iter {
            WhilstOption::ContinueSome(x) => match x.next() {
                Some(x) => Some(WhilstAtom::Continue(x)),
                None => None, // flat map iterator is consumed
            },
            WhilstOption::ContinueNone => None, // flat-map is created on None => empty iterator
            WhilstOption::Stop => Some(WhilstAtom::Stop), // input is Stop
        }
    }
}
