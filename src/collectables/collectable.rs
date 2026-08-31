pub trait Collectable<O>: Extend<O> + Default + FromIterator<O> + Send {}

impl<O: Send> Collectable<O> for alloc::vec::Vec<O> {}

impl<O: Send + Ord> Collectable<O> for alloc::collections::BTreeSet<O> {}
