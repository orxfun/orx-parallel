pub struct VectorUFilterMapIter<I, U, Fm, O>
where
    I: Iterator,
    Fm: Fn(*mut U, I::Item) -> Option<O>,
{
    u: *mut U,
    iter: I,
    filter_map: Fm,
}

impl<I, U, Fm, O> VectorUFilterMapIter<I, U, Fm, O>
where
    I: Iterator,
    Fm: Fn(*mut U, I::Item) -> Option<O>,
{
    pub(crate) fn new(u: *mut U, iter: I, filter_map: Fm) -> Self {
        Self {
            u,
            iter,
            filter_map,
        }
    }
}

impl<I, U, Fm, O> Iterator for VectorUFilterMapIter<I, U, Fm, O>
where
    I: Iterator,
    Fm: Fn(*mut U, I::Item) -> Option<O>,
{
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(x) => {
                    let y = (self.filter_map)(self.u, x);
                    if y.is_some() {
                        return y;
                    }
                }
                None => return None,
            }
        }
    }
}
