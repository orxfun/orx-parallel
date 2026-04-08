use crate::infallible_use::using_var::using::Using;

pub struct UsingClone<T: Clone + Send>(T);

impl<T: Clone + Send> Using for UsingClone<T> {
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
unsafe impl<T: Clone + Send> Sync for UsingClone<T> {}
