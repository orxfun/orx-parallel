pub trait Collectable<O>:
    Extend<O> + Default + FromIterator<O> + IntoIterator<Item = O> + Send
{
    fn len(&self) -> usize;
}

impl<O: Send> Collectable<O> for alloc::vec::Vec<O> {
    fn len(&self) -> usize {
        alloc::vec::Vec::len(self)
    }
}

impl<O: Send + Ord> Collectable<O> for alloc::collections::BTreeSet<O> {
    fn len(&self) -> usize {
        alloc::collections::BTreeSet::len(self)
    }
}
