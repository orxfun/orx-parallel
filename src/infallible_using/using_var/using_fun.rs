use crate::infallible_using::using_var::using::Using;

pub struct UsingFun<T, F: Fn(usize) -> T + Sync>(F);

impl<T, F: Fn(usize) -> T + Sync> UsingFun<T, F> {
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<T, F: Fn(usize) -> T + Sync> Using for UsingFun<T, F> {
    type Item = T;

    #[inline]
    fn create(&self, thread_idx: usize) -> Self::Item {
        (self.0)(thread_idx)
    }

    fn into_inner(self) -> Self::Item {
        (self.0)(0)
    }
}
