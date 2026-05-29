use crate::infallible_use::use_var::r#use::Use;

pub struct UseFun<T, F: Fn(usize) -> T + Sync>(F);

impl<T, F: Fn(usize) -> T + Sync> UseFun<T, F> {
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<T, F: Fn(usize) -> T + Sync> Use for UseFun<T, F> {
    type Item = T;

    #[inline]
    fn create(&self, thread_idx: usize) -> Self::Item {
        (self.0)(thread_idx)
    }
}

impl<T, F: Fn(usize) -> T + Sync> From<F> for UseFun<T, F> {
    fn from(value: F) -> Self {
        Self(value)
    }
}
