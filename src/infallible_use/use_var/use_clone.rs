use crate::infallible_use::use_var::r#use::Use;

pub struct UseClone<T: Clone + Send>(T);

impl<T: Clone + Send> Use for UseClone<T> {
    type Item = T;

    #[inline]
    fn create(&self, _: usize) -> Self::Item {
        self.0.clone()
    }

    #[inline]
    fn into_inner(self) -> Self::Item {
        self.0
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
