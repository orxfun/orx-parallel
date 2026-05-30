use super::r#use::Use;

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
    fn init_get(&self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        (self.0)(thread_idx)
    }

    #[inline]
    fn get(&mut self, thread_idx: usize) -> Self::ItemBorrow<'_> {
        self.init_get(thread_idx)
    }
}

impl<T: Send, F: Fn(usize) -> T + Sync> From<F> for UseFun<T, F> {
    fn from(value: F) -> Self {
        Self(value)
    }
}
