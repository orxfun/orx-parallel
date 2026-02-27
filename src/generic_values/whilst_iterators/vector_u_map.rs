pub struct VectorUMapIter<I, U, M, O>
where
    I: Iterator,
    M: Fn(*mut U, I::Item) -> O,
{
    u: *mut U,
    iter: I,
    map: M,
}

impl<I, U, M, O> VectorUMapIter<I, U, M, O>
where
    I: Iterator,
    M: Fn(*mut U, I::Item) -> O,
{
    pub(crate) fn new(u: *mut U, iter: I, map: M) -> Self {
        Self { u, iter, map }
    }
}

impl<I, U, M, O> Iterator for VectorUMapIter<I, U, M, O>
where
    I: Iterator,
    M: Fn(*mut U, I::Item) -> O,
{
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| (self.map)(self.u, x))
    }
}
