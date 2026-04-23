use crate::infallible_use::Use;
use alloc::boxed::Box;

pub struct UseBox<U>(Box<dyn Use<Item = U>>);

impl<U> Use for UseBox<U> {
    type Item = U;

    #[inline]
    fn create(&self, thread_idx: usize) -> Self::Item {
        self.0.create(thread_idx)
    }
}
