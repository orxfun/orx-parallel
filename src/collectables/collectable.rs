pub trait Collectable<O>:
    Extend<O> + Default + FromIterator<O> + IntoIterator<Item = O> + Send
{
    fn col_len(&self) -> usize;

    fn col_reserve(&mut self, additional: usize);
}

impl<O: Send> Collectable<O> for alloc::vec::Vec<O> {
    fn col_len(&self) -> usize {
        self.len()
    }

    fn col_reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }
}

impl<O: Send + Ord> Collectable<O> for alloc::collections::BTreeSet<O> {
    fn col_len(&self) -> usize {
        self.len()
    }

    fn col_reserve(&mut self, _additional: usize) {}
}
