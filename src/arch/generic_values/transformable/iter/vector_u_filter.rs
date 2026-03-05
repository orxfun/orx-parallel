pub struct VectorUFilterIter<I, U, F>
where
    I: Iterator,
    F: Fn(*mut U, &I::Item) -> bool,
{
    u: *mut U,
    iter: I,
    filter: F,
}

impl<I, U, F> VectorUFilterIter<I, U, F>
where
    I: Iterator,
    F: Fn(*mut U, &I::Item) -> bool,
{
    pub fn new(u: *mut U, iter: I, filter: F) -> Self {
        Self { u, iter, filter }
    }
}

impl<I, U, F> Iterator for VectorUFilterIter<I, U, F>
where
    I: Iterator,
    F: Fn(*mut U, &I::Item) -> bool,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find(|x| (self.filter)(self.u, &x))
    }
}
