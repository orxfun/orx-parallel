use crate::infallible_use::use_var::using::Using;

pub struct UseClone<T: Clone + Send>(T);

impl<T: Clone + Send> Using for UseClone<T> {
    type Item = T;

    type ItemKind<'a>
        = T
    where
        Self: 'a;

    #[inline]
    fn create(&self, _: usize) -> Self::Item {
        self.0.clone()
    }

    #[inline]
    fn get(&self, thread_idx: usize) -> Self::ItemKind<'_> {
        self.create(thread_idx)
    }
}

/// SAFETY: Since T is Send, it is safe to share `UsingClone` with
/// another thread and `create` a clone of `T` on this thread.
unsafe impl<T: Clone + Send> Sync for UseClone<T> {}

impl<T: Clone + Send> UseClone<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: Clone + Send> From<T> for UseClone<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
