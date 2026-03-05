use crate::generic_values::WhilstAtom;

pub struct WhilstVectorMapIter<I, T, O, M>
where
    I: Iterator<Item = WhilstAtom<T>>,
    M: Fn(T) -> O,
{
    iter: I,
    map: M,
}

impl<I, T, O, M> WhilstVectorMapIter<I, T, O, M>
where
    I: Iterator<Item = WhilstAtom<T>>,
    M: Fn(T) -> O,
{
    pub(crate) fn new(iter: I, map: M) -> Self {
        Self { iter, map }
    }
}

impl<I, T, O, M> Iterator for WhilstVectorMapIter<I, T, O, M>
where
    I: Iterator<Item = WhilstAtom<T>>,
    M: Fn(T) -> O,
{
    type Item = WhilstAtom<O>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| match x {
            WhilstAtom::Continue(x) => WhilstAtom::Continue((self.map)(x)),
            WhilstAtom::Stop => WhilstAtom::Stop,
        })
    }
}
