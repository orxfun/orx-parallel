use crate::generic_values::{WhilstAtom, whilst_iterators::WhilstAtomFlatMapIter};

pub struct WhilstVectorFlatMapIter<I, T, Vo, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(T) -> Vo,
    Vo: IntoIterator,
{
    outer: I,
    inner: Option<WhilstAtomFlatMapIter<Vo>>,
    flat_map: Fm,
}

impl<I, T, Vo, Fm> WhilstVectorFlatMapIter<I, T, Vo, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(T) -> Vo,
    Vo: IntoIterator,
{
    pub(crate) fn new(mut outer: I, flat_map: Fm) -> Self {
        let inner = outer
            .next()
            .map(|atom| WhilstAtomFlatMapIter::from_atom(atom, &flat_map));
        Self {
            outer,
            inner,
            flat_map,
        }
    }
}

impl<I, T, Vo, Fm> Iterator for WhilstVectorFlatMapIter<I, T, Vo, Fm>
where
    I: Iterator<Item = WhilstAtom<T>>,
    Fm: Fn(T) -> Vo,
    Vo: IntoIterator,
{
    type Item = WhilstAtom<Vo::Item>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // self.outer.next().map(|x| match x {
        //     WhilstAtom::Continue(x) => WhilstAtom::Continue((self.flat_map)(x)),
        //     WhilstAtom::Stop => WhilstAtom::Stop,
        // })
    }
}
