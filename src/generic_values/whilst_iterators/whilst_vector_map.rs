use crate::generic_values::WhilstAtom;

pub struct WhilstVectorMapIter<I, T, O, M>
where
    I: Iterator<Item = WhilstAtom<T>>,
    M: Fn(T) -> O,
{
    current_iter: I,
    map: M,
}

impl<I, T, O, M> WhilstVectorMapIter<I, T, O, M>
where
    I: Iterator<Item = WhilstAtom<T>>,
    M: Fn(T) -> O,
{
    pub(crate) fn new(current_iter: I, map: M) -> Self {
        Self { current_iter, map }
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
        self.current_iter.next().map(|x| match x {
            WhilstAtom::Continue(x) => WhilstAtom::Continue((self.map)(x)),
            WhilstAtom::Stop => WhilstAtom::Stop,
        })
    }
}
