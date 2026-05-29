use alloc::vec::Vec;

#[derive(Debug)]
pub struct Vec2<T> {
    pub(super) inner: Vec<Vec<T>>,
}

impl<T> Vec2<T> {
    pub fn into_inner(self) -> Vec<Vec<T>> {
        self.inner
    }
}

impl<T> From<Vec<Vec<T>>> for Vec2<T> {
    fn from(inner: Vec<Vec<T>>) -> Self {
        Self { inner }
    }
}

impl<T> From<Vec2<T>> for Vec<Vec<T>> {
    fn from(value: Vec2<T>) -> Self {
        value.inner
    }
}
