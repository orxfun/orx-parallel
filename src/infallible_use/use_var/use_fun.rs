use crate::infallible_use::use_var::r#use::Use;

pub struct UseFun<T, F: Fn(usize) -> T + Sync>(F);

impl<T: Send, F: Fn(usize) -> T + Sync> UseFun<T, F> {
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<T: Send, F: Fn(usize) -> T + Sync> Use for UseFun<T, F> {
    type Item = T;

    type ItemBorrow<'a>
        = T
    where
        Self: 'a;

    #[inline]
    fn create(&self, thread_idx: usize) -> Self::Item {
        (self.0)(thread_idx)
    }

    #[inline]
    fn get(&self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        self.create(thread_idx)
    }

    #[inline]
    fn get_mut(&mut self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        self.get(thread_idx)
    }
}

impl<T: Send, F: Fn(usize) -> T + Sync> From<F> for UseFun<T, F> {
    fn from(value: F) -> Self {
        Self(value)
    }
}
