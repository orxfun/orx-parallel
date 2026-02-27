use crate::generic_values::WhilstAtom;

pub struct WhilstVectorFilterIter<I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(&T) -> bool,
{
    iter: I,
    filter: F,
}

impl<I, T, F> WhilstVectorFilterIter<I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(&T) -> bool,
{
    pub(crate) fn new(iter: I, filter: F) -> Self {
        Self { iter, filter }
    }
}

impl<I, T, F> Iterator for WhilstVectorFilterIter<I, T, F>
where
    I: Iterator<Item = WhilstAtom<T>>,
    F: Fn(&T) -> bool,
{
    type Item = WhilstAtom<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(x) => match x {
                    WhilstAtom::Continue(x) => match (self.filter)(&x) {
                        true => return Some(WhilstAtom::Continue(x)),
                        false => continue,
                    },
                    WhilstAtom::Stop => return Some(WhilstAtom::Stop),
                },
                None => return None,
            }
        }
    }
}
