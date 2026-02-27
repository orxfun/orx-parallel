use crate::generic_values::WhilstAtom;

pub struct WhilstVectorUMapIter<U, I, T, O, M>
where
    I: Iterator<Item = WhilstAtom<T>>,
    M: Fn(*mut U, T) -> O,
{
    u: *mut U,
    iter: I,
    map: M,
}

impl<U, I, T, O, M> WhilstVectorUMapIter<U, I, T, O, M>
where
    I: Iterator<Item = WhilstAtom<T>>,
    M: Fn(*mut U, T) -> O,
{
    pub(crate) fn new(u: *mut U, iter: I, map: M) -> Self {
        Self { u, iter, map }
    }
}

impl<U, I, T, O, M> Iterator for WhilstVectorUMapIter<U, I, T, O, M>
where
    I: Iterator<Item = WhilstAtom<T>>,
    M: Fn(*mut U, T) -> O,
{
    type Item = WhilstAtom<O>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| match x {
            WhilstAtom::Continue(x) => WhilstAtom::Continue((self.map)(self.u, x)),
            WhilstAtom::Stop => WhilstAtom::Stop,
        })
    }
}
