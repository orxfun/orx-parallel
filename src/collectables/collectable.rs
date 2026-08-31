pub trait Collectable<O>: Extend<O> + Default {}

impl<O> Collectable<O> for alloc::vec::Vec<O> {}
